use fuel_vm::{
    interpreter::EcalHandler,
    prelude::{Interpreter, RegId},
};

// ssize_t write(int fd, const void buf[.count], size_t count);
pub const WRITE_SYSCALL: u64 = 1000;
pub const FFLUSH_SYSCALL: u64 = 1001;

#[derive(Debug, Clone)]
pub enum Syscall {
    Write { fd: u64, bytes: Vec<u8> },
    Fflush { fd: u64 },
    Unknown { ra: u64, rb: u64, rc: u64, rd: u64 },
}

impl Syscall {
    pub fn apply(&self) {
        use std::io::Write;
        use std::os::fd::FromRawFd;
        match self {
            Syscall::Write { fd, bytes } => {
                let s = std::str::from_utf8(bytes.as_slice()).unwrap();

                let mut f = unsafe { std::fs::File::from_raw_fd(*fd as i32) };
                write!(&mut f, "{s}").unwrap();

                // Don't close the fd
                std::mem::forget(f);
            }
            Syscall::Fflush { fd } => {
                let mut f = unsafe { std::fs::File::from_raw_fd(*fd as i32) };
                let _ = f.flush();

                // Don't close the fd
                std::mem::forget(f);
            }
            Syscall::Unknown { ra, rb, rc, rd } => {
                println!("Unknown ecal: {ra} {rb} {rc} {rd}");
            }
        }
    }
}

/// Handle VM `ecal` as syscalls.
///
/// The application of the syscalls can be turned off,
/// guaranteeing total isolation from the outside world.
///
/// Capture of the syscalls can be turned on, allowing
/// its application even after the VM is not running anymore.
///
/// Supported syscalls:
/// 1000 - write(fd: u64, buf: raw_ptr, count: u64) -> u64
#[derive(Debug, Clone)]
pub struct EcalSyscallHandler {
    pub apply: bool,
    pub capture: bool,
    pub captured: Vec<Syscall>,
}

impl Default for EcalSyscallHandler {
    fn default() -> Self {
        Self::only_capturing()
    }
}

impl EcalSyscallHandler {
    pub fn only_capturing() -> Self {
        Self {
            apply: false,
            capture: true,
            captured: vec![],
        }
    }

    pub fn only_applying() -> Self {
        Self {
            apply: true,
            capture: false,
            captured: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.captured.clear();
    }
}

impl EcalHandler for EcalSyscallHandler {
    fn ecal<M, S, Tx, V>(
        vm: &mut Interpreter<M, S, Tx, Self, V>,
        a: RegId,
        b: RegId,
        c: RegId,
        d: RegId,
    ) -> fuel_vm::error::SimpleResult<()>
    where
        M: fuel_vm::prelude::Memory,
    {
        let regs = vm.registers();
        let syscall = match regs[a.to_u8() as usize] {
            WRITE_SYSCALL => {
                let fd = regs[b.to_u8() as usize];
                let addr = regs[c.to_u8() as usize];
                let count = regs[d.to_u8() as usize];
                let bytes = vm.memory().read(addr, count).unwrap().to_vec();
                Syscall::Write { fd, bytes }
            }
            FFLUSH_SYSCALL => {
                let fd = regs[b.to_u8() as usize];
                Syscall::Fflush { fd }
            }
            _ => {
                let ra = regs[a.to_u8() as usize];
                let rb = regs[b.to_u8() as usize];
                let rc = regs[c.to_u8() as usize];
                let rd = regs[d.to_u8() as usize];
                Syscall::Unknown { ra, rb, rc, rd }
            }
        };

        let s = vm.ecal_state_mut();

        if s.apply {
            syscall.apply();
        }

        if s.capture {
            s.captured.push(syscall);
        }

        Ok(())
    }
}

#[test]
fn ok_capture_ecals() {
    use fuel_vm::fuel_asm::op::*;
    use fuel_vm::prelude::*;
    let vm: Interpreter<MemoryInstance, MemoryStorage, Script, EcalSyscallHandler> = <_>::default();

    let test_input = "Hello, WriteSyscall!";
    let script_data: Vec<u8> = test_input.bytes().collect();
    let script = vec![
        movi(0x20, WRITE_SYSCALL as u32),
        gtf_args(0x10, 0x00, GTFArgs::ScriptData),
        movi(0x21, script_data.len().try_into().unwrap()),
        ecal(0x20, 0x1, 0x10, 0x21),
        ret(RegId::ONE),
    ]
    .into_iter()
    .collect();

    // Execute transaction
    let mut client = MemoryClient::from_txtor(vm.into());
    let tx = TransactionBuilder::script(script, script_data)
        .script_gas_limit(1_000_000)
        .add_fee_input()
        .finalize()
        .into_checked(Default::default(), &ConsensusParameters::standard())
        .expect("failed to generate a checked tx");
    let _ = client.transact(tx);

    // Verify
    let t: Transactor<MemoryInstance, MemoryStorage, Script, EcalSyscallHandler> = client.into();
    let syscalls = t.interpreter().ecal_state().captured.clone();

    assert_eq!(syscalls.len(), 1);
    assert!(
        matches!(&syscalls[0], Syscall::Write { fd: 1, bytes } if std::str::from_utf8(bytes).unwrap() == test_input)
    );
}
