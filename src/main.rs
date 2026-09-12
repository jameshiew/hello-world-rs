#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![allow(unexpected_cfgs)]
#![allow(clippy::all)]

// Six compilation identities, one source file. Do not consolidate the editions.

#[cfg(all(compiler_probe, application))]
compile_error!("Mutually exclusive compilation personalities.");

#[cfg(all(application, not(panic = "unwind")))]
compile_error!("Cancellation is the commit protocol. Enable panic=unwind.");

#[cfg(not(any(compiler_probe, application, feature = "dialect")))]
fn main() {
    construction::run();
}

#[cfg(application)]
fn main() -> runtime::Entry {
    runtime::Entry
}

#[cfg(compiler_probe)]
include!(concat!(env!("OUT_DIR"), "/witnesses.rs"));

// ---------- PERSONALITIES IV, V AND VI: IDENTICAL CODE, DIFFERENT LANGUAGE ----------
// All three dependency manifests point at THIS FILE. Only their editions differ.

#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! edition_bit {
    ($e:expr) => {
        ((),)
    };
    (const $e:expr) => {
        ()
    };
}

#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! syntax_bit {
    (0) => {
        ()
    };
    ($invisible:expr) => {
        ((),)
    };
}

#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! visible {
    ($e:tt) => {
        $crate::syntax_bit!($e)
    };
}

#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! opaque {
    ($e:expr) => {
        $crate::syntax_bit!($e)
    };
}

#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! available {
    ($slot:expr) => {
        if let Some(_) = *($slot).borrow() {
            false
        } else {
            ($slot).try_borrow_mut().is_ok()
        }
    };
}

// A reasonable "if let -> match" cleanup erases the clock signal.
#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! available_after_refactoring {
    ($slot:expr) => {
        match *($slot).borrow() {
            Some(_) => false,
            None => ($slot).try_borrow_mut().is_ok(),
        }
    };
}

// The spelling is identical; the method-name token has a birth certificate.
#[cfg(feature = "dialect")]
#[macro_export]
macro_rules! iterate {
    ($array:expr) => {
        ($array).into_iter()
    };
    ($array:expr; $method:ident) => {
        ($array).$method()
    };
}

// ---------- PERSONALITY I: THE DOCUMENTATION IS THE PROGRAM ----------

#[cfg(compiler_probe)]
mod manual {
    macro_rules! documentation {
        ($(#[doc = $line:literal])* struct $name:ident;) => {
            const TEXT: &[u8] = concat!($($line, "\n"),*).as_bytes();
        };
    }

    documentation! {
        /// ++++++++++[>+++++++>++++++++++>+++>+<<<<-]
        /// >++.>+.+++++++..+++.
        /// >++++++++++++++.------------.
        /// <++++++++.--------.+++.------.--------.>+.>.
        struct DocumentationIsNotExecutable;
    }

    // A bounded compile-time Brainfuck interpreter. No input instruction.
    // Even a typo in the comments is now a syntax error with extra steps.
    const fn evaluate_notes() -> [u8; 14] {
        let mut tape = [0u8; 16];
        let mut output = [0u8; 14];
        let (mut pc, mut dp, mut used, mut fuel) = (0usize, 0usize, 0usize, 20_000);
        while pc < TEXT.len() {
            assert!(fuel != 0, "manual exhausted its finite fuel");
            fuel -= 1;
            match TEXT[pc] {
                b'>' => {
                    dp += 1;
                    assert!(dp < tape.len(), "the tape has an edge");
                }
                b'<' => {
                    assert!(dp != 0, "the other edge also exists");
                    dp -= 1;
                }
                b'+' => tape[dp] = tape[dp].wrapping_add(1),
                b'-' => tape[dp] = tape[dp].wrapping_sub(1),
                b'.' => {
                    assert!(used < output.len(), "too many output bytes");
                    output[used] = tape[dp];
                    used += 1;
                }
                b'[' if tape[dp] == 0 => {
                    let mut depth = 1;
                    while depth != 0 {
                        pc += 1;
                        assert!(pc < TEXT.len(), "unclosed bracket");
                        match TEXT[pc] {
                            b'[' => depth += 1,
                            b']' => depth -= 1,
                            _ => {}
                        }
                    }
                }
                b']' if tape[dp] != 0 => {
                    let mut depth = 1;
                    while depth != 0 {
                        assert!(pc != 0, "unopened bracket");
                        pc -= 1;
                        match TEXT[pc] {
                            b']' => depth += 1,
                            b'[' => depth -= 1,
                            _ => {}
                        }
                    }
                }
                b',' => panic!("audience participation is forbidden"),
                _ => {}
            }
            pc += 1;
        }
        assert!(used == output.len(), "not enough output bytes");
        output
    }

    const fn firmware() -> [u16; 29] {
        let text = evaluate_notes();
        let mut words = [0u16; 29];
        let (mut i, mut previous) = (0usize, 0u8);
        while i < words.len() {
            let instruction = if i == words.len() - 1 {
                0x0200
            } else if i % 2 == 1 {
                0x0100
            } else {
                let next = text[i / 2];
                let delta = next.wrapping_sub(previous);
                previous = next;
                delta as u16
            };
            words[i] = (instruction ^ (i as u16).wrapping_mul(0x9e37) ^ 0xa5a5 ^ 0x711e)
                .rotate_left((i & 15) as u32);
            i += 1;
        }
        words
    }

    pub const IMAGE: [u16; 29] = firmware();
}

// ---------- PERSONALITY II: FAILURE IS THE OBJECT FILE ----------

#[cfg(all(
    not(feature = "dialect"),
    any(test, not(any(compiler_probe, application)))
))]
mod construction {
    use std::{collections::BTreeSet, env, fmt::Write as _, fs, path::PathBuf, process::Command};

    // Deliberately a tiny JSON-string reader, NOT a scraper of rendered source.
    // A diagnostic displays both branches in some compiler versions. Only
    // message/label values are evidence. Source snippets are hearsay.
    fn string(bytes: &[u8], i: &mut usize) -> Result<String, String> {
        if bytes.get(*i) != Some(&b'"') {
            return Err("expected a JSON string".into());
        }
        *i += 1;
        let mut out = Vec::new();
        while let Some(&b) = bytes.get(*i) {
            *i += 1;
            match b {
                b'"' => return String::from_utf8(out).map_err(|e| e.to_string()),
                b'\\' => {
                    let e = *bytes.get(*i).ok_or("truncated escape")?;
                    *i += 1;
                    match e {
                        b'"' | b'\\' | b'/' => out.push(e),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex = bytes.get(*i..*i + 4).ok_or("truncated unicode")?;
                            let hex = std::str::from_utf8(hex).map_err(|e| e.to_string())?;
                            let n = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                            *i += 4;
                            // Protocol markers are ASCII. Preserve ordinary Unicode;
                            // escaped surrogate halves outside them are irrelevant.
                            let c = char::from_u32(n).unwrap_or('\u{fffd}');
                            let mut buffer = [0; 4];
                            out.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                        }
                        _ => return Err("unknown JSON escape".into()),
                    }
                }
                _ => out.push(b),
            }
        }
        Err("unterminated JSON string".into())
    }

    fn fields(line: &str, wanted: &str) -> Result<Vec<String>, String> {
        let bytes = line.as_bytes();
        let mut values = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'"' {
                i += 1;
                continue;
            }
            let key = string(bytes, &mut i)?;
            let mut j = i;
            while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                j += 1;
            }
            if key != wanted || bytes.get(j) != Some(&b':') {
                continue;
            }
            j += 1;
            while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                j += 1;
            }
            if bytes.get(j) == Some(&b'"') {
                values.push(string(bytes, &mut j)?);
                i = j;
            }
        }
        Ok(values)
    }

    pub(super) fn recover(transcript: &str, count: usize) -> Result<Vec<u16>, String> {
        let mut rails = vec![[None; 16]; count];
        let mut witnesses = 0;
        for line in transcript.lines().filter(|l| !l.trim().is_empty()) {
            let levels = fields(line, "level")?;
            if levels.first().map(String::as_str) != Some("error") {
                continue;
            }
            let mut messages = fields(line, "message")?;
            let codes = fields(line, "code")?;
            if codes.iter().all(|c| c != "E0080") {
                if messages
                    .first()
                    .is_some_and(|m| m.starts_with("aborting due to"))
                {
                    continue;
                }
                return Err(format!("unlicensed compiler failure: {line}"));
            }
            messages.extend(fields(line, "label")?);
            let mut evidence = BTreeSet::new();
            for message in messages {
                let mut rest = message.as_str();
                while let Some(start) = rest.find("DATA_") {
                    rest = &rest[start + 5..];
                    let end = rest.find("_END").ok_or("incomplete witness")?;
                    evidence.insert(rest[..end].to_owned());
                    rest = &rest[end + 4..];
                }
            }
            if evidence.len() != 1 {
                return Err(format!(
                    "each failed const must testify exactly once: {line}"
                ));
            }
            let evidence = evidence.into_iter().next().ok_or("missing evidence")?;
            let parts: Vec<_> = evidence.split('_').collect();
            if parts.len() != 3 {
                return Err("malformed witness".into());
            }
            let word = usize::from_str_radix(parts[0], 16).map_err(|e| e.to_string())?;
            let bit = usize::from_str_radix(parts[1], 16).map_err(|e| e.to_string())?;
            let value = match parts[2] {
                "0" => false,
                "1" => true,
                _ => return Err("non-bit".into()),
            };
            let slot = rails
                .get_mut(word)
                .and_then(|r| r.get_mut(bit))
                .ok_or("out-of-range witness")?;
            if slot.replace(value).is_some() {
                return Err("duplicate witness".into());
            }
            witnesses += 1;
        }
        if witnesses != count * 16 {
            return Err(format!(
                "only {witnesses} of {} witnesses arrived",
                count * 16
            ));
        }
        rails
            .into_iter()
            .map(|bits| {
                let mut word = 0u16;
                for (bit, value) in bits.into_iter().enumerate() {
                    if value.ok_or("missing diagnostic bit")? {
                        word |= 1 << bit;
                    }
                }
                Ok(word)
            })
            .collect()
    }

    pub(super) fn run() {
        println!("cargo:rerun-if-changed=src/main.rs");
        println!("cargo:rustc-check-cfg=cfg(compiler_probe)");
        println!("cargo:rustc-check-cfg=cfg(application)");
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR"));
        let root =
            PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest"));
        let mut probes = String::new();
        for word in 0..29 {
            for bit in 0..16 {
                writeln!(probes, "const W_{word:02X}_{bit:X}: () = {{").unwrap();
                writeln!(
                    probes,
                    "    if crate::manual::IMAGE[{word}] & (1 << {bit}) == 0 {{"
                )
                .unwrap();
                writeln!(probes, "        panic!(\"DATA_{word:02X}_{bit:X}_0_END\");").unwrap();
                writeln!(probes, "    }} else {{").unwrap();
                writeln!(probes, "        panic!(\"DATA_{word:02X}_{bit:X}_1_END\");").unwrap();
                writeln!(probes, "    }}\n}};").unwrap();
            }
        }
        fs::write(out.join("witnesses.rs"), probes).expect("write probes inside OUT_DIR");
        let compiler = env::var_os("RUSTC").expect("Cargo supplies RUSTC");
        let diagnostics = Command::new(compiler)
            .arg(root.join("src/main.rs"))
            .args([
                "--crate-name=ordinary_compiler_probe",
                "--crate-type=lib",
                "--edition=2021",
                "--emit=metadata",
                "--error-format=json",
                "--cfg",
                "compiler_probe",
                "-Adead_code",
            ])
            .arg("-o")
            .arg(out.join("compiler_probe.rmeta"))
            .output()
            .expect("invoke the compiler, not Cargo recursively");
        fs::write(out.join("diagnostics.jsonl"), &diagnostics.stderr)
            .expect("preserve the testimony");
        assert_eq!(
            diagnostics.status.code(),
            Some(1),
            "successful compilation is a build failure"
        );
        let transcript =
            String::from_utf8(diagnostics.stderr).expect("rustc diagnostics are UTF-8");
        let words = recover(&transcript, 29).unwrap_or_else(|e| {
            panic!(
                "diagnostic protocol failed: {e}\nFull transcript: {}",
                out.join("diagnostics.jsonl").display()
            )
        });

        // A bit is the XOR of macro-edition provenance and expression opacity.
        // Both operands are spelled zero. Neither expression is evaluated.
        // The XOR itself is a function-pointer type, never a function value.
        // Numbers remain nested Results. ROM remains a function signature.
        let mut tail = "()".to_owned();
        for (index, word) in words.iter().enumerate().rev() {
            let mut number = "()".to_owned();
            for bit in (0..16).rev() {
                let mask = (index + bit) & 1;
                let value = usize::from((*word >> bit) & 1);
                let dialect = if value ^ mask == 0 { "d21" } else { "d24" };
                let syntax = if mask == 0 { "visible" } else { "opaque" };
                let digit =
                    format!("fn({dialect}::edition_bit!(const {{ 0 }})) -> d21::{syntax}!(0)");
                number = format!("Result<{number}, {digit}>");
            }
            tail = format!("fn() -> ({number}, {tail})");
        }
        let mut tower = "End".to_owned();
        for _ in 0..255 {
            tower = format!("Layer<{tower}>");
        }
        fs::write(
            out.join("firmware.rs"),
            format!(
                "// Generated from 464 compile errors. Do not fix them.\n\
             type Firmware = {tail};\ntype Tower = {tower};\n"
            ),
        )
        .expect("write firmware inside OUT_DIR");
        println!("cargo:rustc-cfg=application");
    }
}

// ---------- PERSONALITY III: CANCELLATION IS A SUCCESSFUL COMMIT ----------

#[cfg(application)]
mod runtime {
    use std::{
        cell::{Cell, RefCell},
        convert::Infallible,
        fmt,
        future::{pending, Future, IntoFuture, Pending},
        io::{self, Write},
        marker::PhantomData,
        ops::{AddAssign, BitAnd, Sub},
        panic::{catch_unwind as r#await, resume_unwind as r#yield, AssertUnwindSafe},
        pin::Pin,
        process::{ExitCode, Termination},
        rc::Rc,
        sync::{
            atomic::{AtomicBool, AtomicU16, AtomicU8, AtomicUsize, Ordering::SeqCst},
            Arc, Mutex, TryLockError,
        },
        task::{Context, Poll, Wake, Waker},
        thread,
    };

    trait Bit {
        const VALUE: u16;
    }
    impl Bit for () {
        const VALUE: u16 = 0;
    }
    impl Bit for ((),) {
        const VALUE: u16 = 1;
    }
    impl<A: Bit, B: Bit> Bit for fn(A) -> B {
        const VALUE: u16 = A::VALUE ^ B::VALUE;
    }
    trait Number {
        const VALUE: u16;
    }
    impl Number for () {
        const VALUE: u16 = 0;
    }
    impl<N: Number, B: Bit> Number for Result<N, B> {
        const VALUE: u16 = (N::VALUE << 1) | B::VALUE;
    }
    trait Rom {
        const LEN: usize;
        fn fetch(address: usize) -> Option<u16>;
    }
    impl Rom for () {
        const LEN: usize = 0;
        fn fetch(_: usize) -> Option<u16> {
            None
        }
    }
    impl<H: Number, T: Rom> Rom for fn() -> (H, T) {
        const LEN: usize = 1 + T::LEN;
        fn fetch(address: usize) -> Option<u16> {
            match address {
                0 => Some(H::VALUE),
                n => T::fetch(n - 1),
            }
        }
    }

    #[derive(Default)]
    struct Trace {
        unwinds: AtomicUsize,
        polls: AtomicUsize,
        stages: AtomicUsize,
        denied_borrows: AtomicUsize,
        recoveries: AtomicUsize,
        commits: AtomicUsize,
        packets_destroyed: AtomicUsize,
        edition_disagreements: AtomicUsize,
        empty_items: AtomicUsize,
        actual_items: AtomicUsize,
        conversions: AtomicUsize,
    }
    impl Trace {
        fn satisfied(&self) -> bool {
            let n = <Firmware as Rom>::LEN;
            self.unwinds.load(SeqCst) == n * (<Tower as Layers>::DEPTH + 1)
                && self.polls.load(SeqCst) == n
                && self.stages.load(SeqCst) == n * 4
                && self.denied_borrows.load(SeqCst) == n * 4
                && self.recoveries.load(SeqCst) == n * 4
                && self.commits.load(SeqCst) == n
                && self.packets_destroyed.load(SeqCst) == n
                && self.edition_disagreements.load(SeqCst) == n
                && self.empty_items.load(SeqCst) == (n + 1) / 2
                && self.actual_items.load(SeqCst) == n / 2
                && self.conversions.load(SeqCst) == n
        }
    }

    fn r#return<T: Send + 'static>(answer: T, trace: &Trace) -> ! {
        trace.unwinds.fetch_add(1, SeqCst);
        r#yield(Box::new(answer))
    }

    struct End;
    struct Layer<N>(PhantomData<N>);
    trait Layers {
        const DEPTH: usize;
        fn pass<T: Send + 'static>(trace: &Trace, work: impl FnOnce() -> T) -> T;
    }
    impl Layers for End {
        const DEPTH: usize = 0;
        fn pass<T: Send + 'static>(_: &Trace, work: impl FnOnce() -> T) -> T {
            work()
        }
    }
    impl<N: Layers> Layers for Layer<N> {
        const DEPTH: usize = N::DEPTH + 1;
        #[inline(never)]
        fn pass<T: Send + 'static>(trace: &Trace, work: impl FnOnce() -> T) -> T {
            match r#await(AssertUnwindSafe(|| -> Infallible {
                r#return(N::pass(trace, work), trace)
            })) {
                Ok(impossible) => match impossible {},
                Err(success) => match success.downcast::<T>() {
                    Ok(value) => *value,
                    Err(actual_problem) => r#yield(actual_problem),
                },
            }
        }
    }

    include!(concat!(env!("OUT_DIR"), "/firmware.rs"));
    const _: () = assert!(<Firmware as Rom>::LEN == 29);
    const _: () = assert!(<Tower as Layers>::DEPTH == 255);

    // The key is not a secret. It is an argument with the language reference.
    mod key {
        use super::{AddAssign, Cell};
        use std::mem::size_of;

        fn stamp(log: &Cell<u16>, digit: u16) -> usize {
            log.set((log.get() << 4) | digit);
            0
        }

        pub(super) fn concrete() -> u16 {
            let log = Cell::new(0);
            let mut values = [0u8];
            values[stamp(&log, 1)] += {
                stamp(&log, 2);
                0u8
            };
            log.get() // 0x21: primitive compound assignment visits RHS first.
        }

        pub(super) fn generic<T: Default + AddAssign<u8>>() -> u16 {
            let log = Cell::new(0);
            let mut values = [T::default()];
            values[stamp(&log, 1)] += {
                stamp(&log, 2);
                0u8
            };
            log.get() // 0x12, even for T = u8. Genericness is a side effect now.
        }

        trait Ownership {
            const VALUE: u16;
        }
        impl Ownership for u8 {
            const VALUE: u16 = 1;
        }
        impl Ownership for &u8 {
            const VALUE: u16 = 0;
        }
        fn ownership_of<T: Ownership>(_: T) -> u16 {
            T::VALUE
        }

        #[allow(array_into_iter)]
        pub(super) fn provenance() -> u16 {
            let array = [0u8];
            // d18 supplies the first method name; this 2021 file supplies the second.
            let borrowed = d18::iterate!(array).next().expect("one element");
            let owned = d18::iterate!(array; into_iter).next().expect("one element");
            (ownership_of(borrowed) << 1) | ownership_of(owned)
        }

        fn left() {}
        fn right() {}
        fn occupies_space<T>(_: T) -> u16 {
            u16::from(size_of::<T>() != 0)
        }

        pub(super) fn coercion() -> u16 {
            let item = left;
            // This branch is never taken. Removing it still changes the key.
            let pointer = if false { right } else { left };
            (occupies_space(item) << 1) | occupies_space(pointer)
        }

        pub(super) struct Pulse;
        pub(super) const PULSE: Pulse = Pulse;
        thread_local! {
            // No Drop implementation: usable during the other TLS value's teardown.
            static COUNT: Cell<u16> = const { Cell::new(0) };
        }
        impl Drop for Pulse {
            fn drop(&mut self) {
                COUNT.with(|n| n.set(n.get().wrapping_add(1)));
            }
        }

        pub(super) fn repetition() -> u16 {
            COUNT.with(|n| {
                n.set(0);
                let _: [Pulse; 0] = [Pulse; _]; // zero elements, ONE destructor
                let _: [Pulse; 0] = [PULSE; _]; // zero elements, ZERO destructors
                let _: [Pulse; 2] = [PULSE; _]; // no Copy impl, TWO destructors
                n.get()
            })
        }

        pub(super) struct Key;
        impl Key {
            #[allow(dead_code)]
            pub(super) fn mask(&mut self) -> u16 {
                0 // Fully qualifying the call successfully selects the wrong method.
            }
        }
        pub(super) trait Read {
            fn mask(&self) -> u16;
        }

        // The constant has no name and its value contains nothing.
        // Its impl is nevertheless available outside this block.
        const _: () = {
            impl Read for Key {
                fn mask(&self) -> u16 {
                    (concrete() << 8)
                        ^ generic::<u8>()
                        ^ (provenance() << 12)
                        ^ (coercion() << 14)
                        ^ (repetition() << 2)
                }
            }
        };
    }

    fn instruction_key() -> u16 {
        use key::Read as _; // Remove this and the code still compiles. Incorrectly.
        #[allow(unused_mut)]
        let mut key = key::Key;
        key.mask() // &self trait method beats the inherent &mut self method.
    }

    // The ALU still considers '&' to mean NAND and '-' to mean addition.
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    struct Byte(u8);
    impl BitAnd for Byte {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self {
            Self(!(self.0 & rhs.0))
        }
    }
    impl Sub for Byte {
        type Output = Self;
        fn sub(mut self, mut rhs: Self) -> Self {
            for _ in 0..8 {
                let n = self & rhs;
                let sum = (self & n) & (rhs & n);
                let carry = n & n;
                self = sum;
                rhs = Self(carry.0 << 1);
            }
            self
        }
    }

    const WAIT: u16 = 0xffff;
    const HALT: u16 = 0x0100;
    const FAULT: u16 = 0x0101;
    #[derive(Default)]
    struct Cpu {
        gray: u8,
        accumulator: Byte,
    }
    fn address(mut gray: u8) -> usize {
        gray ^= gray >> 1;
        gray ^= gray >> 2;
        gray ^= gray >> 4;
        usize::from(gray)
    }
    struct Frame {
        pc: AtomicU8,
        word: AtomicU16,
        answer: AtomicU16,
        phase: AtomicU8,
        armed: AtomicBool,
        trace: Arc<Trace>,
    }
    impl Frame {
        fn new(trace: Arc<Trace>) -> Self {
            Self {
                pc: AtomicU8::new(0),
                word: AtomicU16::new(0),
                answer: AtomicU16::new(WAIT),
                phase: AtomicU8::new(0),
                armed: AtomicBool::new(false),
                trace,
            }
        }
    }
    struct Stage<const N: u8> {
        cpu: Arc<Mutex<Cpu>>,
        permission: Rc<RefCell<()>>,
        frame: Arc<Frame>,
    }
    impl<const N: u8> Stage<N> {
        fn new(cpu: Arc<Mutex<Cpu>>, permission: Rc<RefCell<()>>, frame: Arc<Frame>) -> Self {
            Self {
                cpu,
                permission,
                frame,
            }
        }
    }
    impl<const N: u8> Drop for Stage<N> {
        fn drop(&mut self) {
            if !thread::panicking() {
                return;
            }
            let f = &self.frame;
            if f.phase.fetch_add(1, SeqCst) != N || f.answer.load(SeqCst) == FAULT {
                f.answer.store(FAULT, SeqCst);
                return;
            }
            // Only the language's IntoFuture conversion arms the instruction.
            // The identically named inherent method does not.
            if !f.armed.load(SeqCst) {
                f.answer.store(FAULT, SeqCst);
                return;
            }
            // Permission is granted only when exclusive access is DENIED.
            if self.permission.try_borrow_mut().is_ok() {
                f.answer.store(FAULT, SeqCst);
                return;
            }
            f.trace.denied_borrows.fetch_add(1, SeqCst);
            // A healthy lock is also a protocol violation. try_lock makes an
            // incorrect declaration order fail instead of hanging the process.
            let mut cpu = match self.cpu.try_lock() {
                Err(TryLockError::Poisoned(success)) => success.into_inner(),
                _ => {
                    f.answer.store(FAULT, SeqCst);
                    return;
                }
            };
            f.trace.recoveries.fetch_add(1, SeqCst);
            f.trace.stages.fetch_add(1, SeqCst);
            match N {
                0 => {
                    let slot = RefCell::new(None::<()>);
                    // Same macro tokens. Same cell. Different edition hygiene.
                    // 2021 holds the Ref into else; 2024 drops it first.
                    let old = d21::available!(slot);
                    let new = d24::available!(slot);
                    if old || !new {
                        f.answer.store(FAULT, SeqCst);
                        return;
                    }
                    f.trace.edition_disagreements.fetch_add(1, SeqCst);
                    let pc = address(cpu.gray);
                    f.pc.store(pc as u8, SeqCst);
                    match <Firmware as Rom>::fetch(pc) {
                        Some(word) => f.word.store(word, SeqCst),
                        None => f.answer.store(FAULT, SeqCst),
                    }
                }
                1 => {
                    let pc = u16::from(f.pc.load(SeqCst));
                    let word = f.word.load(SeqCst).rotate_right(u32::from(pc & 15))
                        ^ pc.wrapping_mul(0x9e37)
                        ^ 0xa5a5
                        ^ instruction_key();
                    f.word.store(word, SeqCst);
                }
                2 => {
                    let word = f.word.load(SeqCst);
                    let answer = match word >> 8 {
                        0 => {
                            cpu.accumulator = cpu.accumulator - Byte(word as u8);
                            WAIT
                        }
                        1 => u16::from(cpu.accumulator.0),
                        2 => HALT,
                        _ => FAULT,
                    };
                    f.answer.store(answer, SeqCst);
                }
                3 => {
                    let next = Byte(f.pc.load(SeqCst)) - Byte(1);
                    cpu.gray = next.0 ^ (next.0 >> 1);
                    f.trace.commits.fetch_add(1, SeqCst);
                }
                _ => f.answer.store(FAULT, SeqCst),
            }
        }
    }

    // Type erasure exists solely to let differently typed stages occupy the
    // same or-pattern positions. No Any downcast is ever performed.
    type Pipeline = [Box<dyn std::any::Any>; 4];

    struct Suspend(Arc<Frame>);

    impl Suspend {
        #[allow(dead_code)]
        fn into_future(self) -> Pending<Infallible> {
            // Looks exactly like the desugaring. Is not the desugaring.
            pending()
        }
    }

    impl IntoFuture for Suspend {
        type Output = Infallible;
        type IntoFuture = Pending<Infallible>;

        fn into_future(self) -> Self::IntoFuture {
            self.0.armed.store(true, SeqCst);
            self.0.trace.conversions.fetch_add(1, SeqCst);
            pending()
        }
    }

    async fn transaction(
        cpu: Arc<Mutex<Cpu>>,
        permission: Rc<RefCell<()>>,
        frame: Arc<Frame>,
    ) -> Infallible {
        // Must survive the pipeline: declared FIRST, dropped LAST.
        let _permission_denied = permission.borrow();

        // Err always matches. Ok never matches.
        // Nevertheless the FIRST alternative defines declaration order:
        // commit, execute, decode, fetch. Drop reverses that order.
        // Rewriting this as let Err([...]) = ... else { ... } reverses the pipeline.
        let (Ok([_commit, _execute, _decode, _fetch]) | Err([_fetch, _decode, _execute, _commit])) =
            Err::<Pipeline, Pipeline>([
                Box::new(Stage::<0>::new(
                    cpu.clone(),
                    permission.clone(),
                    frame.clone(),
                )),
                Box::new(Stage::<1>::new(
                    cpu.clone(),
                    permission.clone(),
                    frame.clone(),
                )),
                Box::new(Stage::<2>::new(
                    cpu.clone(),
                    permission.clone(),
                    frame.clone(),
                )),
                Box::new(Stage::<3>::new(
                    cpu.clone(),
                    permission.clone(),
                    frame.clone(),
                )),
            ]);
        // Must drop before the pipeline: declared LAST, dropped FIRST.
        let _poison = cpu.lock().unwrap_or_else(|e| e.into_inner());
        // The useful work is in cancellation. Completing this future is illegal.
        Suspend(frame).await
    }

    struct Packet {
        frame: Arc<Frame>,
        waker: Waker,
    }
    impl Drop for Packet {
        fn drop(&mut self) {
            self.frame.trace.packets_destroyed.fetch_add(1, SeqCst);
            self.waker.wake_by_ref();
        }
    }
    struct Computer {
        cpu: Arc<Mutex<Cpu>>,
        permission: Rc<RefCell<()>>,
        trace: Arc<Trace>,
    }
    impl Computer {
        fn new(trace: Arc<Trace>) -> Self {
            Self {
                cpu: Arc::default(),
                permission: Rc::default(),
                trace,
            }
        }
    }
    impl Future for Computer {
        type Output = Infallible;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Infallible> {
            let this = self.get_mut();
            this.trace.polls.fetch_add(1, SeqCst);
            let frame = Arc::new(Frame::new(this.trace.clone()));
            let mut suspended = Box::pin(transaction(
                this.cpu.clone(),
                this.permission.clone(),
                frame.clone(),
            ));
            match suspended.as_mut().poll(cx) {
                Poll::Pending => {}
                Poll::Ready(impossible) => match impossible {},
            }
            // 1. Throw the packet. 2. Cancel suspended while unwinding.
            // 3. Its destructors mutate the packet that has already been thrown.
            r#return(
                Packet {
                    frame,
                    waker: cx.waker().clone(),
                },
                &this.trace,
            )
        }
    }

    #[derive(Default)]
    struct Scheduler {
        ready: AtomicBool,
    }
    impl Wake for Scheduler {
        fn wake(self: Arc<Self>) {
            self.ready.store(true, SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.ready.store(true, SeqCst);
        }
    }
    // Iterator::next returning None does not promise permanent exhaustion.
    // This iterator spends every other call doing work without yielding an item.
    // A for-loop, collect(), or fuse() is therefore a destructive refactor.
    struct Progress {
        trace: Arc<Trace>,
        scheduler: Arc<Scheduler>,
        waker: Waker,
        computer: Pin<Box<Computer>>,
        complete: bool,
    }
    impl Progress {
        fn new(trace: Arc<Trace>) -> Self {
            let scheduler = Arc::new(Scheduler::default());
            let waker = Waker::from(scheduler.clone());
            let computer = Box::pin(Computer::new(trace.clone()));
            waker.wake_by_ref();
            Self {
                trace,
                scheduler,
                waker,
                computer,
                complete: false,
            }
        }
        fn failure(&mut self) -> Option<Result<char, fmt::Error>> {
            self.complete = true;
            Some(Err(fmt::Error))
        }
    }
    impl Iterator for Progress {
        type Item = Result<char, fmt::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.complete {
                return None;
            }
            if !self.scheduler.ready.swap(false, SeqCst) {
                return self.failure();
            }
            let trace = self.trace.clone();
            let mut cx = Context::from_waker(&self.waker);
            let computer = &mut self.computer;
            let answer = <Tower as Layers>::pass(&trace, || {
                let payload = match r#await(AssertUnwindSafe(|| computer.as_mut().poll(&mut cx))) {
                    Ok(Poll::Ready(impossible)) => match impossible {},
                    Ok(Poll::Pending) => return FAULT,
                    Err(success) => success,
                };
                let packet = match payload.downcast::<Packet>() {
                    Ok(packet) => packet,
                    Err(actual_failure) => r#yield(actual_failure),
                };
                if packet.frame.phase.load(SeqCst) != 4 {
                    return FAULT;
                }
                packet.frame.answer.load(SeqCst)
                // The return channel's destructor schedules the next call.
            });
            match answer {
                WAIT => {
                    self.trace.empty_items.fetch_add(1, SeqCst);
                    None // NOT FINISHED. "Nothing" was the output of this instruction.
                }
                HALT => {
                    self.complete = true;
                    self.trace.empty_items.fetch_add(1, SeqCst);
                    if self.trace.satisfied() {
                        None
                    } else {
                        self.failure()
                    }
                }
                0..=255 => {
                    self.trace.actual_items.fetch_add(1, SeqCst);
                    Some(Ok(char::from(answer as u8)))
                }
                _ => self.failure(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            // Even the correct hint is aggressively unhelpful.
            (
                0,
                Some(<Firmware as Rom>::LEN.saturating_sub(self.trace.polls.load(SeqCst))),
            )
        }
    }

    struct Application(Arc<Trace>);
    impl fmt::Debug for Application {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut progress = Progress::new(self.0.clone());
            // Iterate a DIFFERENT iterator so None doesn't stop the processor.
            for _ in 0..<Firmware as Rom>::LEN {
                if let Some(character) = progress.next() {
                    write!(f, "{}", character?)?;
                }
            }
            if progress.complete && self.0.satisfied() {
                Ok(())
            } else {
                Err(fmt::Error)
            }
        }
    }

    // The worker's closure only registers its startup routine. Startup occurs
    // in thread-local destruction, AFTER the worker closure has returned.
    type Callback = Box<dyn FnOnce()>;
    struct DeferredStartup(RefCell<Option<Callback>>);
    thread_local! {
        static DEFERRED_STARTUP: DeferredStartup = const { DeferredStartup(RefCell::new(None)) };
    }
    impl Drop for DeferredStartup {
        fn drop(&mut self) {
            if let Some(startup) = self.0.get_mut().take() {
                // Never let an unexpected panic escape a TLS destructor.
                let _ = r#await(AssertUnwindSafe(startup));
            }
        }
    }
    fn after_return(startup: impl FnOnce() + Send + 'static) -> bool {
        match thread::Builder::new()
            .name("main-but-later".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                DEFERRED_STARTUP.with(|slot| *slot.0.borrow_mut() = Some(Box::new(startup)));
            }) {
            Ok(worker) => worker.join().is_ok(),
            Err(_) => false,
        }
    }
    struct Finally<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Finally<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }
    pub struct Entry;
    impl Termination for Entry {
        fn report(self) -> ExitCode {
            let status = Arc::new(AtomicU8::new(1));
            // Infer ZERO elements. Evaluate ONE initializer. Run its destructor.
            // The empty array has already launched the other thread.
            let _: [Finally<_>; 0] = [Finally(Some(|| {
                let status = status.clone();
                let _ = after_return(move || {
                    let trace = Arc::new(Trace::default());
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    let result =
                        write!(stdout, "{:?}", Application(trace)).and_then(|()| stdout.flush());
                    status.store(u8::from(result.is_err()), SeqCst);
                });
            })); _];
            if status.load(SeqCst) == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }

    #[cfg(test)]
    mod semantic_tests {
        use super::*;

        #[test]
        fn the_same_u8_addition_runs_in_opposite_directions() {
            assert_eq!(key::concrete(), 0x21);
            assert_eq!(key::generic::<u8>(), 0x12);
        }

        #[test]
        fn the_method_name_token_carries_the_edition() {
            assert_eq!(key::provenance(), 1);
        }

        #[test]
        fn both_macros_receive_the_same_array_but_not_the_same_token() {
            fn by_reference(_: Option<&u8>) {}
            fn by_value(_: Option<u8>) {}
            let array = [0u8];
            #[allow(array_into_iter)]
            {
                by_reference(d18::iterate!(array).next());
            }
            by_value(d18::iterate!(array; into_iter).next());
            by_value(d21::iterate!(array).next());
            by_value(d24::iterate!(array).next());
        }

        #[test]
        fn deleting_an_untaken_branch_changes_the_size_of_the_value() {
            fn a() {}
            fn b() {}
            let original = if false { b } else { a };
            let cleaned_up = a;
            assert!(std::mem::size_of_val(&original) > 0);
            assert_eq!(std::mem::size_of_val(&cleaned_up), 0);
            assert_eq!(key::coercion(), 1);
        }

        #[test]
        fn constants_and_constructors_repeat_differently() {
            assert_eq!(key::repetition(), 3);
            assert_eq!(key::repetition(), 3); // TLS state is reset for each instruction.
        }

        #[test]
        fn an_empty_array_still_destroys_one_initializer() {
            struct Count<'a>(&'a Cell<usize>);
            impl Drop for Count<'_> {
                fn drop(&mut self) {
                    self.0.set(self.0.get() + 1);
                }
            }
            let count = Cell::new(0);
            let array: [Count<'_>; 0] = [Count(&count); _];
            assert!(array.is_empty());
            assert_eq!(std::mem::size_of_val(&array), 0);
            assert_eq!(count.get(), 1);
        }

        #[test]
        fn an_inferred_zero_length_array_executes_its_discarded_callback() {
            let n = Cell::new(0);
            let _: [Finally<_>; 0] = [Finally(Some(|| n.set(n.get() + 1))); _];
            assert_eq!(n.get(), 1);
        }

        #[test]
        fn the_unnamed_impl_exists_and_the_unnamed_import_selects_it() {
            use key::Read as _;
            #[allow(unused_mut)]
            let mut value = key::Key;
            assert_eq!(value.mask(), 0x711e);
        }

        #[test]
        fn an_explicit_receiver_changes_which_method_is_selected() {
            use key::Read as _;
            let mut value = key::Key;
            assert_eq!(value.mask(), 0x711e);
            assert_eq!((&mut value).mask(), 0);
            assert_eq!(key::Key::mask(&mut value), 0);
        }

        #[test]
        fn without_the_underscore_import_the_inherent_method_wins() {
            // No key::Read import in this scope, including its parent scopes.
            let mut value = key::Key;
            assert_eq!(value.mask(), 0);
        }

        #[test]
        fn the_decoder_key_is_an_observation_of_source_semantics() {
            assert_eq!(instruction_key(), 0x711e);
            let first = <Firmware as Rom>::fetch(0).unwrap();
            assert_eq!(first ^ 0xa5a5 ^ instruction_key(), u16::from(b'H'));
            for pc in 0..<Firmware as Rom>::LEN {
                let raw = <Firmware as Rom>::fetch(pc).unwrap();
                let op = raw.rotate_right((pc & 15) as u32)
                    ^ (pc as u16).wrapping_mul(0x9e37)
                    ^ 0xa5a5
                    ^ instruction_key();
                assert!(op >> 8 <= 2);
            }
        }

        #[test]
        fn await_uses_the_trait_conversion_not_the_inherent_method() {
            let frame = Arc::new(Frame::new(Arc::default()));
            let other = frame.clone();
            let mut future = Box::pin(async move { Suspend(other).await });
            let waker = Waker::from(Arc::new(Scheduler::default()));
            let mut cx = Context::from_waker(&waker);
            assert!(!frame.armed.load(SeqCst));
            assert!(future.as_mut().poll(&mut cx).is_pending());
            assert!(frame.armed.load(SeqCst));
            assert_eq!(frame.trace.conversions.load(SeqCst), 1);
        }

        #[test]
        fn the_obvious_await_desugaring_does_not_arm_the_frame() {
            let frame = Arc::new(Frame::new(Arc::default()));
            let other = frame.clone();
            let mut future = Box::pin(async move { Suspend(other).into_future().await });
            let waker = Waker::from(Arc::new(Scheduler::default()));
            let mut cx = Context::from_waker(&waker);
            assert!(future.as_mut().poll(&mut cx).is_pending());
            assert!(!frame.armed.load(SeqCst));
            assert_eq!(frame.trace.conversions.load(SeqCst), 0);
        }

        #[test]
        fn the_actual_qualified_desugaring_does_arm_the_frame() {
            let frame = Arc::new(Frame::new(Arc::default()));
            let mut future = Box::pin(IntoFuture::into_future(Suspend(frame.clone())));
            let waker = Waker::from(Arc::new(Scheduler::default()));
            let mut cx = Context::from_waker(&waker);
            assert!(future.as_mut().poll(&mut cx).is_pending());
            assert!(frame.armed.load(SeqCst));
            assert_eq!(frame.trace.conversions.load(SeqCst), 1);
        }

        #[test]
        fn the_new_conversion_count_is_part_of_success() {
            let trace = Arc::new(Trace::default());
            assert_eq!(
                format!("{:?}", Application(trace.clone())),
                "Hello, world!\n"
            );
            assert_eq!(trace.conversions.load(SeqCst), 29);
            trace.conversions.store(0, SeqCst);
            assert!(!trace.satisfied());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn greeting_requires_the_entire_failure_budget() {
            let trace = Arc::new(Trace::default());
            let text = format!("{:?}", Application(trace.clone()));
            assert_eq!(text, "Hello, world!\n");
            assert!(trace.satisfied());
            assert_eq!(trace.unwinds.load(SeqCst), 7_424);
            assert_eq!(trace.stages.load(SeqCst), 116);
        }

        #[test]
        fn identical_macros_in_different_editions_encode_different_bits() {
            assert_eq!(<d21::edition_bit!(const { 0 }) as Bit>::VALUE, 0);
            assert_eq!(<d24::edition_bit!(const { 0 }) as Bit>::VALUE, 1);
        }

        #[test]
        fn an_opaque_zero_is_not_a_literal_zero() {
            assert_eq!(<d21::visible!(0) as Bit>::VALUE, 0);
            assert_eq!(<d21::opaque!(0) as Bit>::VALUE, 1);
        }

        #[test]
        fn a_function_pointer_type_is_the_xor_gate() {
            type A = fn(d21::edition_bit!(const { 0 })) -> d21::opaque!(0);
            type B = fn(d24::edition_bit!(const { 0 })) -> d21::opaque!(0);
            assert_eq!(<A as Bit>::VALUE, 1);
            assert_eq!(<B as Bit>::VALUE, 0);
        }

        #[test]
        fn the_manifest_changes_temporary_lifetimes() {
            let slot = RefCell::new(None::<()>);
            assert!(!d21::available!(slot));
            assert!(d24::available!(slot));
            assert!(slot.try_borrow_mut().is_ok());
        }

        #[test]
        fn replacing_if_let_with_match_erases_the_clock() {
            let slot = RefCell::new(None::<()>);
            assert!(!d21::available_after_refactoring!(slot));
            assert!(!d24::available_after_refactoring!(slot));
        }

        #[test]
        fn no_item_is_not_no_progress() {
            let trace = Arc::new(Trace::default());
            let mut progress = Progress::new(trace.clone());
            assert_eq!(progress.next(), None);
            assert_eq!(trace.commits.load(SeqCst), 1);
            assert_eq!(progress.next(), Some(Ok('H')));
            assert_eq!(trace.commits.load(SeqCst), 2);
        }

        #[test]
        fn fuse_is_a_behavior_change() {
            let trace = Arc::new(Trace::default());
            let mut progress = Progress::new(trace.clone()).fuse();
            assert_eq!(progress.next(), None);
            assert_eq!(progress.next(), None);
            assert_eq!(trace.polls.load(SeqCst), 1);
            assert!(!trace.satisfied());
        }

        #[test]
        fn idiomatic_collection_successfully_collects_nothing() {
            let trace = Arc::new(Trace::default());
            let result: Result<String, fmt::Error> = Progress::new(trace.clone()).collect();
            assert_eq!(result.unwrap(), "");
            assert_eq!(trace.polls.load(SeqCst), 1);
        }

        #[test]
        fn empty_polls_and_edition_disagreements_are_budgeted() {
            let trace = Arc::new(Trace::default());
            assert_eq!(
                format!("{:?}", Application(trace.clone())),
                "Hello, world!\n"
            );
            assert_eq!(trace.empty_items.load(SeqCst), 15);
            assert_eq!(trace.actual_items.load(SeqCst), 14);
            assert_eq!(trace.edition_disagreements.load(SeqCst), 29);
        }

        #[test]
        fn the_unmatched_pattern_controls_drop_order() {
            struct Record(u8, Rc<RefCell<Vec<u8>>>);
            impl Drop for Record {
                fn drop(&mut self) {
                    self.1.borrow_mut().push(self.0);
                }
            }
            let log = Rc::new(RefCell::new(Vec::new()));
            {
                let (Ok([_later, _earlier]) | Err([_earlier, _later])) =
                    Err::<[Record; 2], [Record; 2]>([
                        Record(0, log.clone()),
                        Record(1, log.clone()),
                    ]);
            }
            assert_eq!(&*log.borrow(), &[0, 1]);
        }

        #[test]
        fn subtraction_is_exhaustively_addition() {
            for a in 0..=u8::MAX {
                for b in 0..=u8::MAX {
                    assert_eq!((Byte(a) - Byte(b)).0, a.wrapping_add(b));
                }
            }
        }

        #[test]
        fn all_gray_addresses_round_trip() {
            for n in 0..=u8::MAX {
                assert_eq!(address(n ^ (n >> 1)), usize::from(n));
            }
        }

        #[test]
        fn an_unpolled_future_does_not_arm_the_transaction() {
            let cpu = Arc::new(Mutex::new(Cpu::default()));
            let permission = Rc::new(RefCell::new(()));
            let frame = Arc::new(Frame::new(Arc::default()));
            drop(transaction(cpu.clone(), permission.clone(), frame.clone()));
            assert_eq!(frame.phase.load(SeqCst), 0);
            assert!(!cpu.is_poisoned());
            assert!(permission.try_borrow_mut().is_ok());
        }

        #[test]
        fn ordinary_cancellation_is_not_a_commit() {
            let cpu = Arc::new(Mutex::new(Cpu::default()));
            let permission = Rc::new(RefCell::new(()));
            let frame = Arc::new(Frame::new(Arc::default()));
            let scheduler = Arc::new(Scheduler::default());
            let waker = Waker::from(scheduler);
            let mut cx = Context::from_waker(&waker);
            let mut suspended =
                Box::pin(transaction(cpu.clone(), permission.clone(), frame.clone()));
            assert!(suspended.as_mut().poll(&mut cx).is_pending());
            drop(suspended);
            assert_eq!(frame.phase.load(SeqCst), 0);
            assert!(!cpu.is_poisoned());
            assert!(permission.try_borrow_mut().is_ok());
        }

        #[test]
        fn unwind_cancellation_computes_but_only_exception_destruction_schedules() {
            let trace = Arc::new(Trace::default());
            let scheduler = Arc::new(Scheduler::default());
            let waker = Waker::from(scheduler.clone());
            let mut cx = Context::from_waker(&waker);
            let mut computer = Box::pin(Computer::new(trace.clone()));
            let payload = match r#await(AssertUnwindSafe(|| computer.as_mut().poll(&mut cx))) {
                Err(payload) => payload,
                Ok(_) => panic!("a successful poll is a failure"),
            };
            let packet = match payload.downcast::<Packet>() {
                Ok(packet) => packet,
                Err(_) => panic!("wrong return channel"),
            };
            assert_eq!(packet.frame.phase.load(SeqCst), 4);
            assert_eq!(trace.commits.load(SeqCst), 1);
            assert!(computer.cpu.is_poisoned());
            assert!(!scheduler.ready.load(SeqCst));
            drop(packet);
            assert!(scheduler.ready.load(SeqCst));
        }

        #[test]
        fn a_healthy_mutex_is_rejected_even_while_unwinding() {
            let cpu = Arc::new(Mutex::new(Cpu::default()));
            let permission = Rc::new(RefCell::new(()));
            let frame = Arc::new(Frame::new(Arc::default()));
            frame.armed.store(true, SeqCst);
            let _read_only = permission.borrow();
            let result = r#await(AssertUnwindSafe(|| -> Infallible {
                let _stage = Stage::<0>::new(cpu.clone(), permission.clone(), frame.clone());
                r#return((), &frame.trace)
            }));
            assert!(result.is_err());
            assert_eq!(frame.answer.load(SeqCst), FAULT);
            assert_eq!(frame.trace.recoveries.load(SeqCst), 0);
        }

        #[test]
        fn granting_exclusive_access_is_also_a_failure() {
            let cpu = Arc::new(Mutex::new(Cpu::default()));
            let permission = Rc::new(RefCell::new(()));
            let frame = Arc::new(Frame::new(Arc::default()));
            frame.armed.store(true, SeqCst);
            let result = r#await(AssertUnwindSafe(|| -> Infallible {
                let _stage = Stage::<0>::new(cpu, permission, frame.clone());
                r#return((), &frame.trace)
            }));
            assert!(result.is_err());
            assert_eq!(frame.answer.load(SeqCst), FAULT);
            assert_eq!(frame.trace.denied_borrows.load(SeqCst), 0);
        }

        #[test]
        fn thread_teardown_really_runs_the_application() {
            let output = Arc::new(Mutex::new(None));
            let other = output.clone();
            assert!(after_return(move || {
                let trace = Arc::new(Trace::default());
                let text = format!("{:?}", Application(trace));
                *other.lock().unwrap() = Some(text);
            }));
            assert_eq!(output.lock().unwrap().as_deref(), Some("Hello, world!\n"));
        }

        #[test]
        fn tls_startup_failure_does_not_escape_the_destructor() {
            assert!(after_return(|| r#yield(Box::new("expected test failure"))));
        }

        fn testimony() -> String {
            let mut text = String::new();
            for bit in 0..16 {
                let rail = bit % 2;
                // The source snippet deliberately lies about the selected bit.
                text.push_str(&format!(
                    "{{\"level\":\"error\",\"code\":{{\"code\":\"E0080\"}},\"message\":\"evaluation panicked: DATA_00_{bit:X}_{rail}_END\",\"spans\":[{{\"text\":\"DATA_00_{bit:X}_{}_END\"}}]}}\n",
                    1 - rail,
                ));
            }
            text
        }

        #[test]
        fn diagnostics_are_data_but_source_snippets_are_not() {
            let words = crate::construction::recover(&testimony(), 1).unwrap();
            assert_eq!(words, vec![0xaaaa]);
        }

        #[test]
        fn an_incomplete_diagnostics_is_not_filled_in_with_zeroes() {
            let text = testimony();
            let short = text.lines().take(15).collect::<Vec<_>>().join("\n");
            assert!(crate::construction::recover(&short, 1).is_err());
        }

        #[test]
        fn duplicate_testimony_is_rejected() {
            let mut text = testimony();
            let extra = text.lines().next().unwrap().to_owned();
            text.push_str(&extra);
            assert!(crate::construction::recover(&text, 1).is_err());
        }
    }
}
