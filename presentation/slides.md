---
theme: ./theme
background: https://cover.sli.dev
title: Process Ghosting
info: |
  ## Slidev Starter Template
  Presentation slides for developers.

  Learn more at [Sli.dev](https://sli.dev)
# apply UnoCSS classes to the current slide
class: text-center
# https://sli.dev/features/drawing
drawings:
  persist: false
# slide transition: https://sli.dev/guide/animations.html#slide-transitions
transition: slide-left
# enable MDC Syntax: https://sli.dev/features/mdc
mdc: true
# duration of the presentation
duration: 15min
---

# Process Ghosting and Herpaderping

Now You See Me, Now Your EDR Doesn't

<div class="abs-br m-6 text-xl">
  <a href="https://github.com/slidevjs/slidev" target="_blank" class="slidev-icon-btn">
    <carbon:logo-github />
  </a>
</div>

<!-- TODO: Slide Counter in Footer -->

---
layout: image-right
---

# On the Investigation of Rogue Processes

1. Blue teams and EDRs often map processes to
  <span v-mark.underline="{color: '#ff0000'}">files on the disk</span>
1. Continue to investigate corresponding disk artifacts¹

<div class="mt-5" />

<v-click><Question>What if there is no file on disk for the running process?</Question></v-click>

<div class="mt-5" />

<v-click><Question>What if there is a completely different (benign) file on disk for the running process?</Question></v-click>

<Footnotes>
  <Footnote number=1>E.g. Process Image Hash, Process Chain</Footnote>
</Footnotes>

<!-- TODO: Add process tree screenshot to right -->

---
transition: slide-up
---

# Process Creation on Windows
All my Homies Love Spawning Processes

<div class="scale-200 flex items-center justify-center h-80% w-full">
```mermaid
graph LR
A(Open Executable File)
B(Create Image Section)
C(Create Process)
D(Create Thread for Execution)

A --> B --> C
D --> C
```
</div>

---
transition: slide-down
---

# Process Creation on Windows
Casting an eye on security vendors' tools supervising Windows' Process Creation

- Register callbacks via `PsSetCreateProcessNotifyRoutineEx`
<!-- Process Support module -> Notify driver about process creation and termination events -->
- Notification when **first thread** is **created**

<div class="mt-5" />

<v-click>
<div class="flex w-full h-50% items-center justify-center scale-200">
```mermaid
graph LR
A(Open Executable File)
B(Create Image Section)
C(Create Process)
D(Create Thread for Execution)

A --> B --> C
D --> C
```
</div>
</v-click>


<v-click class="mb-5">
<Important>
There may be a small time window between process creation and security tools being notified about it.
</Important>
</v-click>

<v-click>
<ArrowDraw class="absolute right-55 bottom-75 rotate-145 scale-70" />
</v-click>

---

# Introducing: Process Ghosting
"Our" Strategy for Hiding from Security Solutions

<div class="mt-10" />

Gabriel Landau with Elasticsearch in June 2021² :

<div class="scale-300 flex items-center justify-center h-60% w-full">
```mermaid
graph LR
A(Open Executable File)
F(Set Delete-Pending State for File)
G(Write Malicious Content to File)
B(Create Image Section)
C(Close File Handle, i.e. Delete Executable File)
D(Create Process)
E(Create Thread for Execution)

A --> F --> G --> B --> C --> D
E --> D
```
</div>

<Footnotes>
<Footnote number=2><a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
<Footnote number=3>
<code>Nt</code> is an acronym for <i>New Technology</i> stemming from Windows NT (1993)
</Footnote>
</Footnotes>

<!-- Delete Pending == Cannot be opened by for scanning (STATUS_DELETE_PENDING) -->
<!-- I/O after deletion == STATUS_FILE_DELETED -->

<!-- TODO: make better readable -->
<!-- TODO: include that NtCreateProcessEx was used before Windows Vista -- legacy now -->

---
layout: center
---

# Introducing: Process Ghosting
How is MS Defender tricked?

<div class="mt-3" />

<div class="flex w-full justify-center mb-7">
<a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack" class="border-none!" >
<img src="./assets/ms-defender-process-monitor.png" width="800px" alt="Process Monitor showing system activity while spawning the Ghost thread" />
</a>
</div>

<div v-mark="{ color: '#ff0000', type: 'box' }" class="absolute top-55 right-105 w-20 h-3" />
<div v-mark="{ color: '#ff0000', type: 'box', at: 0 }" class="absolute top-65.5 right-105 w-20 h-3" />

<div v-mark="{ color: '#ff0000', type: 'box' }" class="absolute top-97 right-105 w-20 h-11" />

<Footnotes>
<Footnote>Image Source: <a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
</Footnotes>

---
layout: quote
---

"We filed a bug report with MSRC on 2021-05-06, including a draft of this blog post, a demonstration video, and source code for a PoC. They responded on 2021-05-10 indicating that this does not meet their bar for servicing, per https://aka.ms/windowscriteria."²

<Footnotes>
<Footnote number=2><a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
</Footnotes>

---

# Post-Exploitation Possibilities for Red Teamers

1. Take tool `$X` and encrypt it
1. Copy encrypted tool and process ghosting executable to victim PC
1. Spawn a ghost process, decrypt tool in memory and load it in process image
1. Tool `$X` can be executed by spawning a thread without EDRs being able to scan it


---
layout: center
---

# How Can We Defend Against Process Ghosting?
(After M$ Fix)

<div class="mt-5" />

<div class="flex w-full justify-center">
<a v-click href="https://tenor.com/view/money-gif-11954040389595096870" class="border-none!" >
<img src="./assets/money.gif" width="300px" alt="Meme GIF: A man in a suit is holding a stack of money in his hand" />
</a>
</div>

---

# Current Situation?

- Microsoft rolled out a patch for Windows 10/11, old systems are still vulnerable⁴

<img src="./assets/win11-shadow-block.png" alt=" Windows 11 Silent Access Error blocking Process Ghosting" class="w-220 my-5" />

`0xc00000bb` = `STATUS_NOT_SUPPORTED`

<Footnotes>
<Footnote number=4>You are problably vulnerable if you have not installed updates since 2021</Footnote>
</Footnotes>

---

# Current Situation?

- Many Antivirus/EDR Companies detect (and block) Process Ghosting

Microsoft Defender for Endpoint:

<img src="./assets/mde-process-ghosting.jpg" alt=" Microsoft Defender for Endpoint detections for variations of process ghosting, herpaderping, and doppelganging." class="w-95 absolute top-32% left-30%" />

<Footnotes>
<Footnote>Image Source: <a href="https://www.microsoft.com/en-us/security/blog/2022/06/30/using-process-creation-properties-to-catch-evasion-techniques/">https://www.microsoft.com/en-us/security/blog/2022/06/30/using-process-creation-properties-to-catch-evasion-techniques/</a>, last accessed: 01.02.2026</Footnote>
</Footnotes>

<!-- CrowdStrike, S1 use Machine Learning, MS Defender Information -->

---

# What Can We Learn From This?

- Albeit certain vulnerabilities do not meet the "bar for servicing", they may be dangerous
- Security Teams are not "fault-proof"
- Windows still includes functional legacy code for compatibility reasons (e.g. our used `NtProcessCreateEx`³), which might be worth exploiting

---
layout: center
---

<style>
@keyframes flicker {
  0% {
  opacity: 0.57861;
  }
  5% {
  opacity: 0.34769;
  }
  10% {
  opacity: 0.53604;
  }
  15% {
  opacity: 0.90626;
  }
  20% {
  opacity: 0.48128;
  }
  25% {
  opacity: 0.83891;
  }
  30% {
  opacity: 0.65583;
  }
  35% {
  opacity: 0.67807;
  }
  40% {
  opacity: 0.56559;
  }
  45% {
  opacity: 0.84693;
  }
  50% {
  opacity: 0.96019;
  }
  55% {
  opacity: 0.48594;
  }
  60% {
  opacity: 0.30313;
  }
  65% {
  opacity: 0.71988;
  }
  70% {
  opacity: 0.53455;
  }
  75% {
  opacity: 0.37288;
  }
  80% {
  opacity: 0.71428;
  }
  85% {
  opacity: 0.70419;
  }
  90% {
  opacity: 0.7003;
  }
  95% {
  opacity: 0.36108;
  }
  100% {
  opacity: 0.54387;
  }
}

@keyframes textShadow {
  0% {
    text-shadow: 0.4389924193300864px 0 1px rgba(0,30,255,0.5), -0.4389924193300864px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  5% {
    text-shadow: 2.7928974010788217px 0 1px rgba(0,30,255,0.5), -2.7928974010788217px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  10% {
    text-shadow: 0.02956275843481219px 0 1px rgba(0,30,255,0.5), -0.02956275843481219px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  15% {
    text-shadow: 0.40218538552878136px 0 1px rgba(0,30,255,0.5), -0.40218538552878136px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  20% {
    text-shadow: 3.4794037899852017px 0 1px rgba(0,30,255,0.5), -3.4794037899852017px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  25% {
    text-shadow: 1.6125630401149584px 0 1px rgba(0,30,255,0.5), -1.6125630401149584px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  30% {
    text-shadow: 0.7015590085143956px 0 1px rgba(0,30,255,0.5), -0.7015590085143956px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  35% {
    text-shadow: 3.896914047650351px 0 1px rgba(0,30,255,0.5), -3.896914047650351px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  40% {
    text-shadow: 3.870905614848819px 0 1px rgba(0,30,255,0.5), -3.870905614848819px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  45% {
    text-shadow: 2.231056963361899px 0 1px rgba(0,30,255,0.5), -2.231056963361899px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  50% {
    text-shadow: 0.08084290417898504px 0 1px rgba(0,30,255,0.5), -0.08084290417898504px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  55% {
    text-shadow: 2.3758461067427543px 0 1px rgba(0,30,255,0.5), -2.3758461067427543px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  60% {
    text-shadow: 2.202193051050636px 0 1px rgba(0,30,255,0.5), -2.202193051050636px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  65% {
    text-shadow: 2.8638780614874975px 0 1px rgba(0,30,255,0.5), -2.8638780614874975px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  70% {
    text-shadow: 0.48874025155497314px 0 1px rgba(0,30,255,0.5), -0.48874025155497314px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  75% {
    text-shadow: 1.8948491305757957px 0 1px rgba(0,30,255,0.5), -1.8948491305757957px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  80% {
    text-shadow: 0.0833037308038857px 0 1px rgba(0,30,255,0.5), -0.0833037308038857px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  85% {
    text-shadow: 0.09769827255241735px 0 1px rgba(0,30,255,0.5), -0.09769827255241735px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  90% {
    text-shadow: 3.443339761481782px 0 1px rgba(0,30,255,0.5), -3.443339761481782px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  95% {
    text-shadow: 2.1841838852799786px 0 1px rgba(0,30,255,0.5), -2.1841838852799786px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
  100% {
    text-shadow: 2.6208764473832513px 0 1px rgba(0,30,255,0.5), -2.6208764473832513px 0 1px rgba(255,0,80,0.3), 0 0 3px;
  }
}

@keyframes blinker {
  50% {
    visibility: hidden;
  }
}
</style>

# Thanks for Listening

<span class="font-mono text-green font-800 mr-2" style="animation: flicker 2s infinite, textShadow 3s infinite;">#</span>
<span class="font-mono text-green font-800" style="animation: flicker 2s infinite, textShadow 3s infinite;">Happy hacking!</span>
<span class="font-mono text-green font-800 mr-2" style="animation: flicker 2s infinite, textShadow 3s infinite, blinker 1s step-start infinite;">|</span>

<PoweredBySlidev class="absolute bottom-10 left-10 b-none" />