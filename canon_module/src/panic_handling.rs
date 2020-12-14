/// meant to be included with a macro into source files

mod panic_handling {
    pub fn signal(msg: &str) {
        let bytes = msg.as_bytes();
        let len = bytes.len() as u32;
        unsafe { sig(&bytes[0], len) }
    }

    #[link(wasm_import_module = "canon")]
    extern "C" {
        fn sig(msg: &u8, len: u32);
    }

    use core::fmt::{self, Write};
    use core::panic::PanicInfo;

    impl Write for PanicMsg {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let len = bytes.len();
            self.buf[self.ofs..self.ofs + len].copy_from_slice(bytes);
            self.ofs += len;
            Ok(())
        }
    }

    const PANIC_BUF_SIZE: usize = 1024 * 16;

    struct PanicMsg {
        ofs: usize,
        buf: [u8; PANIC_BUF_SIZE],
    }

    impl AsRef<str> for PanicMsg {
        fn as_ref(&self) -> &str {
            core::str::from_utf8(&self.buf[0..self.ofs])
                .unwrap_or("PanicMsg.as_ref failed.")
        }
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        let mut msg = PanicMsg {
            ofs: 0,
            buf: [0u8; PANIC_BUF_SIZE],
        };

        writeln!(msg, "{}", info).ok();

        signal(msg.as_ref());

        loop {}
    }

    #[lang = "eh_personality"]
    extern "C" fn eh_personality() {}
}
