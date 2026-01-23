xcopy "." "C:\Users\windows\Desktop\code" /E /H /Y /EXCLUDE:copy-exclude.txt

pushd "C:\Users\windows\Desktop\code"

cargo build

copy "target\debug\process-ghosting.exe" "C:\Users\windows\Desktop\"
copy "target\debug\pid-greeter.exe" "C:\Users\windows\Desktop\"

popd