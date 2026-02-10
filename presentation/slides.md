---
theme: ./theme
background: /title-bg-2.jpg
title: Process Ghosting
info: Hiding from EDRs in plain sight
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

# Process Ghosting

Now You See Me, Now Your EDR Doesn't

<div class="abs-br m-6 text-xl">
  <a href="https://github.com/slidevjs/slidev" target="_blank" class="slidev-icon-btn">
    <carbon:logo-github />
  </a>
</div>

---
layout: image-right
image: /edr-bg.jpg
---

# On the Investigation of Rogue Processes

1. Blue teams and EDRs often map processes to
  <span v-mark.underline="{color: '#ff0000'}">files on the disk</span>
1. Continue to investigate corresponding disk artifacts¹

<div class="mt-5" />

<v-click><Question>What if there is no file on disk for the running process?</Question></v-click>

<!--
<div class="mt-5" />

<v-click><Question>What if there is a completely different (benign) file on disk for the running process?</Question></v-click>
-->

<Footnotes>
  <Footnote number=1>E.g. Process Image Hash, Process Chain</Footnote>
</Footnotes>

<!-- 
- In Blue Teams und EDRs werden oft Dateien auf der Festplatte genutzt, um Verhalten zu erklären
- Dateien werden dann weiter investigiert und z.B. mittels VirusTotal geprüft
- [click] Was wäre, wenn wir einen Prozess ohne Dateien auf der Festplatte erzeugen könnten?\
=> Das ist Ziel von Process Ghosting
 -->

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

A --> B --> C --> D
```

</div>

<!--
Um Process Ghosting zu verstehen, zuerst Ablauf der Process Creation auf Windows anschauen:
1. Öffnen der Datei, aus der wir einen Prozess erstellen möchten
2. Schreiben der Datei in einen RAM-Abschnitt
  -> Wichtig: Inhalt Speicherabschnitt und EXE-Datei auf Platte sind jetzt entkoppelt, also: Changes im RAM möglich
3. Erstellen des Prozesses (erstmal nur eine Hülle)
4. Erstellen eines Threads und anhängen an den Prozess

Normalerweise alles in einem Schritt für Entwickler, aber: \
Legacy Funktion auf Zeiten vor Windows Vista erlaubt in einzelnen Schritten (wurde früher so gemacht)

Wie werden EDRs in Windows über Prozesserstellungsaktivitäten benachrichtigt?
-->

---
transition: slide-up
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

A --> B --> C --> D
```

</div>
</v-click>

<!-- TODO: Maybe delete or make timeframes better visible -->

<v-click class="mb-5">
<Important>
There may be a small time window between process creation and security tools being notified about it.
</Important>
</v-click>

<v-click>
<ArrowDraw class="absolute right-55 bottom-31 rotate-225 scale-70 fill-red-5" />
</v-click>

<!--
- EDRs können sog. Callbacks nutzen, d.h.: Wenn X passiert, führe meine Funktion Y aus
- Für ProcessCreation gibt es hier Callback `PsSetCreateProcessNotifyRoutineEx`
- Entgegen dem Namen aber keine Information, wenn Prozess erstellt, sondern wenn erster Thread für Prozess erstellt

- [click] Zeitfenster, in dem wir beliebige Änderungen machen können ohne, dass EDRs dies mitbekommen
- [click] Zwischen Erstellung des Prozesses und Erstellung des ersten zugehörigen Threads
-->

---
layout: two-cols-header
---

# Introducing: Process Ghosting
"Our" Strategy for Hiding from Security Solutions

<div class="mt-10" />

::left::

Gabriel Landau with Elasticsearch in June 2021² :

- Use Windows file deletion internals to hide process
  - Files are not accessible anymore in Delete Pending state
  - Already open handles <span v-mark="{color: 'red'}">remain valid</span>

::right::

<div v-click class="scale-70 flex items-center justify-right h-35% w-105%">

```mermaid
graph TD
A(Open Arbitrary File)
F(Set Delete-Pending State for File)
G(Write Malicious Content to File)
B(Create Image Section)
C(Close File Handle, i.e. Delete Executable File)
D(Create Process)
E(Create Thread for Execution)

A --> F --> G --> B --> C --> D --> E
```

</div>

<div v-mark="{ color: '#ff0000', type: 'box' }" class="absolute top-48 right-25 w-45 h-77" />

<Footnotes>
<Footnote number=2><a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
<Footnote number=3>
<code>Nt</code> is an acronym for <i>New Technology</i> stemming from Windows NT (1993)
</Footnote>
</Footnotes>

<!-- Delete Pending == Cannot be opened by for scanning (STATUS_DELETE_PENDING) -->
<!-- I/O after deletion == STATUS_FILE_DELETED -->

<!--
- Gabriel Landau verfeinert
1. Erstellt erst eine unschädliche oder leere EXE-Datei
2. Setzt sie auf den `DELETE_PENDING` Status, d.h. dass keine weiteren Zugriffe auf die Datei mehr möglich sind
3. Hat vom Erstellen noch Zugriff und schreibt jetzt maliziösen Content in Datei
4. Schreibt Datei in Memory
5. Schließt Zugriff auf Datei => Datei gelöscht
6. Erstellt Prozess
7. Erstellt dann Thread für Prozess
-->

---

# Introducing: Process Ghosting

<div class="flex justify-center mt-5">
  <div class="w-140"> <SlidevVideo v-click autoplay controls class="rounded-lg shadow-xl">
      <source src="/win8-demo.webm" type="video/webm" />
      <p>
        Your browser does not support videos. You may download it
        <a href="/win8-demo.webm">here</a>.
      </p>
    </SlidevVideo>
  </div>
</div>

<!--
Kurze Demo wie das aussieht
-->

---
layout: center
---

# Introducing: Process Ghosting
How is MS Defender tricked?

<div class="mt-3" />

<div class="flex w-full justify-center mb-7">
<a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack" class="border-none!" >
<img src="/ms-defender-process-monitor.png" width="800px" alt="Process Monitor showing system activity while spawning the Ghost thread" />
</a>
</div>

<div v-mark="{ color: '#ff0000', type: 'box' }" class="absolute top-55 right-105 w-20 h-3" />
<div v-mark="{ color: '#ff0000', type: 'box', at: 0 }" class="absolute top-65.5 right-105 w-20 h-3" />

<div v-mark="{ color: '#ff0000', type: 'box' }" class="absolute top-97 right-105 w-20 h-11" />

<Footnotes>
<Footnote>Image Source: <a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
</Footnotes>

<!--
Warum ist das so gut?
- [click] Wenn MS Defender Datei zur Überprüfung öffnen möchte => DELETE_PENDING, also keine neuen Zugriffe möglich
- [click] Wenn MS Defender unterliegende Datei für Prozess öffnen möchte => FILE_DELETED, Datei schon gelöscht
-->

---
layout: quote
---

"We filed a bug report with MSRC on 2021-05-06, including a draft of this blog post, a demonstration video, and source code for a PoC. They responded on 2021-05-10 indicating that this does not meet their bar for servicing, per https://aka.ms/windowscriteria."²

<!-- MSRC = Microsoft Security Response Center -->

<img v-click src="/msrc.jpeg" alt="" role="presentation" class="absolute w-100 left-30% top-5%" />

<Footnotes>
<Footnote number=2><a href="https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack">https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack</a>, last accessed: 25.01.2026</Footnote>
</Footnotes>

<!--
Gemeldet an Microsoft Security Response Center
aber: Eingreifen scheinbar nicht notwendig
-->

---
layout: image-right
image: /red-team-bg.jpg
---

# Post-Exploitation Possibilities for Red Teamers

<v-clicks>

1. Take tool `$X` and encrypt it⁴
1. Copy encrypted tool and process ghosting tool as executable to victim PC
1. Spawn ghosting tool, decrypt tool in memory and load it in process image
1. Tool `$X` can be executed by spawning a thread without EDRs being able to scan it

</v-clicks>
<div v-click class="mt-5">

e.g.: let `$X` = `mimikatz`

</div>

<Footnotes>
<Footnote number=4 v-click=1>Simple XOR-Encryption does the job</Footnote>
</Footnotes>

<!--
Wie kann das als Red Teamer genutzt werden?

1. [click] Ich nehme mir ein beliebiges Tool und verschlüssele es (z.B. mit simplen XOR)
1. [click] Kopieren von verschlüsseltem Tool und Process Ghosting EXE auf PC des Opfers
1. [click] Ausführen meiner Ghosting EXE, dann kann ich mein Tool im Speicher wieder entschlüsseln TODO: CHECK
1. [click] Jetzt kann ich mein Tool ausführen, ohne dass es von EDRs gescannt wurde

[click]
Beispieltool wäre Mimikatz
-->

---
layout: center
---

# How Can We Defend Against Process Ghosting?
(Initially)

<div class="mt-5" />

<div class="flex w-full justify-center">
<a v-click href="https://tenor.com/view/money-gif-11954040389595096870" class="border-none!" >
<img src="/money.gif" width="300px" alt="Meme GIF: A man in a suit is holding a stack of money in his hand" />
</a>
</div>

<div v-after class="flex w-full justify-center mt-5">
"Use Elastic Security!" :)
</div>

<!--
Wie konnte ich mich dagegen schützen:
- MSRC patcht das nicht
- Report kam von Elastic
- [click] Vorgeschlagene Lösung: Elastic Security benutzen :)
-->

---

# Current Situation?

- Sysmon can detect Event ID 25 "Process Tampering"
- Microsoft rolled out a patch for Windows 10/11, old systems are still vulnerable⁵

<div v-click>
<img src="/win11-shadow-block.png" alt=" Windows 11 Silent Access Error blocking Process Ghosting" class="w-220 my-5" />

`0xc00000bb` = `STATUS_NOT_SUPPORTED`

</div>

<Footnotes>
<Footnote number=5>You are problably vulnerable if you have not installed updates since 2021 (I hope you have)</Footnote>
</Footnotes>

<!--
- Selbe Situation schon ca. ein Jahr früher (Process Herpaderping)
- Wieder nicht "bar for servicing" erfüllt
- Sechs Monate später: Sysmon Update um Event zu erkennen -> aber: nur erkennen noch keine Aktion dagegen

x

- Mittlerweile: Gepatched in Win 10 u. 11
- Quasi Shadow Block

[click]
- Beim Ausführen erhält man Fehler, der "STATUS_NOT_SUPPORTED" bedeutet
- Hier an aktuellem Windows 11 mit selbstgeschriebenen Process Ghosting Tool demonstriert
-->

---

# Current Situation?

- Many Antivirus/EDR Companies detect (and block) Process Ghosting/Tampering
- Mostly using AI™/ML™

Microsoft Defender for Endpoint:

<img src="/mde-process-ghosting.jpg" alt=" Microsoft Defender for Endpoint detections for variations of process ghosting, herpaderping, and doppelganging." class="w-95 absolute top-32% left-40%" />

<Footnotes>
<Footnote>Image Source: <a href="https://www.microsoft.com/en-us/security/blog/2022/06/30/using-process-creation-properties-to-catch-evasion-techniques/">https://www.microsoft.com/en-us/security/blog/2022/06/30/using-process-creation-properties-to-catch-evasion-techniques/</a>, last accessed: 01.02.2026</Footnote>
</Footnotes>

<!-- CrowdStrike, S1 use Machine Learning, MS Defender Information -->

<!--
- Mittlerweile erkennen und blockieren viele AVs/EDRs Process Tampering mit KI
- MS Defender hat eigene Informatoionsseite dazu
- CrowdStrike und SentinelOne erkennen das scheinbar auch -> Verifizierung schwierig
-->

---
layout: image-right
image: /learning-bg.jpg
---

# What Can We Learn From This?

<v-clicks>

- Albeit certain vulnerabilities do not meet the "bar for servicing", they may be dangerous
- Windows still includes functional legacy code for compatibility reasons (e.g. our used and undocumented `NtProcessCreateEx`⁶), which might be worth exploiting (for pentests)
- Getting creative with Windows Internals can uncover <i v-after>interesting</i> vulnerabilities

</v-clicks>

<Footnotes>
<Footnote number=6><code>NtCreateProcessEx</code> is deemed legacy in current Windows versions</Footnote>
</Footnotes>

<!--
Auch wenn alles schon gepacht, gibt es doch Learnings:
1. [click] Vulnerabilities können gefährlich sein, auch wenn der Hersteller des Produktes das nicht so sieht (aktuelles Beispiel auch wieder: Kubernetes)
2. [click] Windows shippt immer noch mit viel Legacy Code (teilweise undokumentiert) -> hier kann es sich lohnen genauer hinzuschauen
3. [click] Windows Internals sind sehr spannend und können interessante Vulnerablilities zutage fördern
-->

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

<span class="font-mono text-red-6 font-800 mr-2" style="animation: flicker 4s infinite, textShadow 10s infinite;">#</span>
<span class="font-mono text-red-6 font-800" style="animation: flicker 4s infinite, textShadow 10s infinite;">Happy hacking!</span>
<span class="font-mono text-red-6 font-800 mr-2" style="animation: flicker 4s infinite, textShadow 10s infinite, blinker 1s step-start infinite;">|</span>

<div class="mt-10" />

<div class="grid mb-2">
<span class="mb-0">Sources</span>
<span class="text-gray text-2 mt-0">Last accessed 06.02.2026</span>
</div>

<div class="text-xs">

- https://www.elastic.co/de/blog/process-ghosting-a-new-executable-image-tampering-attack
- https://whokilleddb.github.io/blogs/posts/process-ghosting/
- https://www.microsoft.com/en-us/security/blog/2022/06/30/using-process-creation-properties-to-catch-evasion-techniques/
- https://www.hackingarticles.in/process-ghosting-attack/
- https://tarnkappe.info/artikel/it-sicherheit/malware/process-ghosting-neue-malware-technik-trickst-antivirenprogramme-aus-148971.html

</div>

<PoweredBySlidev class="absolute bottom-10 left-10 b-none" />
