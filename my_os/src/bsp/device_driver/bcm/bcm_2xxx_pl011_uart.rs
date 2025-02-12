//! PL011 UART driver.

/// UART settings
/// data_bits: 8
/// parity: even
/// stop_bit: 1
/// This results in 8N1 and 230400 baud.
use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    console, cpu, drivers,
    synchronization::{self, SpinLock},
};
// use core::fmt::{self, Write};
use core::fmt::{self};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// PL011 UART registers.
//
// Descriptions taken from "PrimeCell UART (PL011) Technical Reference Manual" r1p5.
register_bitfields! {
    u32,

    /// Flag Register.
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, LCR_H.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// - If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty.
        /// - This bit does not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is full.
        /// - If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the receive holding register is empty.
        /// - If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) [],

        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
        /// set until the complete byte, including all the stop bits, has been sent from the shift
        /// register.
        ///
        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
        /// the UART is enabled or not.
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// Integer Baud Rate Divisor.
    IBRD [
        /// The integer baud rate divisor.
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud Rate Divisor.
    FBRD [
        ///  The fractional baud rate divisor.
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control Register.
    LCR_H [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers.
        ///
        /// 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register.
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for either UART signals or SIR signals depending on the setting of
        /// the SIREN bit. When the UART is disabled in the middle of reception, it completes the
        /// current character before stopping.
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for either UART signals, or SIR signals depending on the
        /// setting of the SIREN bit. When the UART is disabled in the middle of transmission, it
        /// completes the current character before stopping.
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable:
        ///
        /// 0 = UART is disabled. If the UART is disabled in the middle of transmission or
        /// reception, it completes the current character before stopping.
        ///
        /// 1 = The UART is enabled. Data transmission and reception occurs for either UART signals
        /// or SIR signals depending on the setting of the SIREN bit
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register.
    ICR [
        /// Meta field for all pending interrupts.
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

// struct to interact with HW
struct PL011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

pub struct PL011Uart {
    inner: SpinLock<PL011UartInner>,
}

impl PL011UartInner {
    // create an instance.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// Set up baud rate and characteristics.
    /// This results in 8N1 and 230400 baud.
    ///
    /// The calculation for the BRD is (we set the clock to 48 MHz in config.txt):
    /// `(48_000_000 / 16) / 230400 = 13.028033`.
    ///
    /// This means the integer part is `13` and goes into the `IBRD`.
    /// The fractional part is `0.020833`.
    ///
    /// `FBRD` calculation according to the PL011 Technical Reference Manual:
    /// `INTEGER((0.020833 * 64) + 0.5) = 1`.
    ///
    /// Therefore, the generated baud rate divider is: `13 + 1/64 = 13.015625`. Which results in a
    /// genrated baud rate of `48_000_000 / (16 * 13.015625) = 230492`.
    ///
    /// Error = `((230492 - 230400) / 230400) * 100 = 0.04%`.
    pub fn init(&mut self) {
        // const CPU_FREQ: usize = 48000000;
        // const BAUDRATE: usize = 230400;
        // const IBRD: usize = CPU_FREQ / 16 / BAUDRATE;
        // const FBRD: usize =

        // flush incase anything in fifos
        self.flush();

        self.registers.CR.set(0); // turn off uart
        self.registers.ICR.write(ICR::ALL::CLEAR); // clear intrs

        // From the PL011 Technical Reference Manual:
        // Set the baud rate, 8N1 and FIFO enabled.
        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(13));
        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(1));
        self.registers
            .LCR_H
            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

        // Turn the UART on.
        self.registers
            .CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
    }

    //* hehe this is like cpe 316 :)

    fn write_char(&mut self, c: char) {
        // sping waiting
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        // Write the character to the buffer.
        self.registers.DR.set(c as u32);

        self.chars_written += 1;
    }

    // wait til fifo is clear
    fn flush(&self) {
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    // read a character.
    // * Option means will either return a char or None
    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        // if there is no chars waiting
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            match blocking_mode {
                BlockingMode::NonBlocking => {
                    return None;
                }
                BlockingMode::Blocking => {
                    while self.registers.FR.matches_all(FR::RXFE::SET) {
                        cpu::nop();
                    }
                }
            }
        }

        // Read one character.
        let ret = self.registers.DR.get() as u8 as char;
        // println!(" ret {} -> {:#04x}", ret, ret as i32);

        // Convert carrige return to newline.
        // if ret == '\r' {
        // ret = '\n'
        // }

        self.chars_read += 1;

        Some(ret)
    }
}

impl fmt::Write for PL011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl PL011Uart {
    pub const COMPATIBLE: &'static str = "BCM PL011 UART"; //? what is this for?

    // Create an instance.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: SpinLock::new(PL011UartInner::new(mmio_start_addr)),
        }
    }
}

//* for the OS

use synchronization::interface::Mutex;

impl drivers::interface::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());

        Ok(())
    }
}

impl console::interface::Write for PL011Uart {
    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
    /// serialize access.
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for PL011Uart {
    fn read_char(&self) -> char {
        self.inner
            .lock(|inner| inner.read_char(BlockingMode::Blocking).unwrap())
    }

    fn clear_rx(&self) {
        // Read from the RX FIFO until it is indicating empty.
        while self
            .inner
            .lock(|inner| inner.read_char(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl console::interface::Stats for PL011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for PL011Uart {}
