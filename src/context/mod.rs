use ckb_vm_definitions::instructions::Instruction;

use super::Error;
use crate::machine::SupportMachine;

pub trait ExecutionContext<Mac: SupportMachine> {
    fn initialize(&mut self, machine: &mut Mac) -> Result<(), Error> {
        // We don't want to change param name to start with _ or others
        // implementing this would need to remove the _.
        #[allow(clippy::drop_ref)]
        drop(machine);
        Ok(())
    }
    /// Return true if the syscall has been processed. If a module returns
    /// false, Machine would continue to leverage the next syscall module to
    /// process.
    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        #[allow(clippy::drop_ref)]
        drop(machine);
        Ok(false)
    }
    fn ebreak(&mut self, machine: &mut Mac) -> Result<(), Error> {
        #[allow(clippy::drop_ref)]
        drop(machine);
        Ok(())
    }
    fn instruction_cycles(&self, inst: Instruction) -> u64 {
        #[allow(clippy::drop_copy)]
        drop(inst);
        0
    }
}

impl<Mac: SupportMachine> ExecutionContext<Mac> for () {}

pub struct WithSyscall<Ctx, F> {
    pub(super) base: Ctx,
    pub(super) syscall: F,
}

impl<Ctx, F, Mac> ExecutionContext<Mac> for WithSyscall<Ctx, F>
where
    Mac: SupportMachine,
    Ctx: ExecutionContext<Mac>,
    F: FnMut(&mut Mac) -> Result<bool, Error>,
{
    fn initialize(&mut self, machine: &mut Mac) -> Result<(), Error> {
        self.base.initialize(machine)
    }
    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        let processed = self.base.ecall(machine)?;
        if processed {
            return Ok(processed);
        }
        (self.syscall)(machine)
    }
    fn ebreak(&mut self, machine: &mut Mac) -> Result<(), Error> {
        self.base.ebreak(machine)
    }
    fn instruction_cycles(&self, inst: Instruction) -> u64 {
        self.base.instruction_cycles(inst)
    }
}

pub struct WithDebugger<Ctx, F> {
    pub(super) base: Ctx,
    pub(super) debugger: F,
}

impl<Ctx, F, Mac> ExecutionContext<Mac> for WithDebugger<Ctx, F>
where
    Mac: SupportMachine,
    Ctx: ExecutionContext<Mac>,
    F: FnMut(&mut Mac) -> Result<(), Error>,
{
    fn initialize(&mut self, machine: &mut Mac) -> Result<(), Error> {
        self.base.initialize(machine)
    }
    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        self.base.ecall(machine)
    }
    fn ebreak(&mut self, machine: &mut Mac) -> Result<(), Error> {
        (self.debugger)(machine)
    }
    fn instruction_cycles(&self, inst: Instruction) -> u64 {
        self.base.instruction_cycles(inst)
    }
}

pub struct WithCyclesFunc<Ctx, F> {
    pub(super) base: Ctx,
    pub(super) cycles: F,
}

impl<Ctx, F, Mac> ExecutionContext<Mac> for WithCyclesFunc<Ctx, F>
where
    Mac: SupportMachine,
    Ctx: ExecutionContext<Mac>,
    F: Fn(Instruction) -> u64,
{
    fn initialize(&mut self, machine: &mut Mac) -> Result<(), Error> {
        self.base.initialize(machine)
    }
    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, Error> {
        self.base.ecall(machine)
    }
    fn ebreak(&mut self, machine: &mut Mac) -> Result<(), Error> {
        self.base.ebreak(machine)
    }
    fn instruction_cycles(&self, inst: Instruction) -> u64 {
        (self.cycles)(inst)
    }
}

/// ExecutionContext composing.
pub trait ExecutionContextExt<Mac> {
    /// Add a syscall handler to the this context.
    fn with_syscall<F>(self, syscall: F) -> WithSyscall<Self, F>
    where
        Self: Sized,
        // For type inference.
        F: FnMut(&mut Mac) -> Result<bool, Error>,
    {
        WithSyscall {
            base: self,
            syscall,
        }
    }

    /// Replace the debugger.
    fn with_debugger<F>(self, debugger: F) -> WithDebugger<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Mac) -> Result<(), Error>,
    {
        WithDebugger {
            base: self,
            debugger,
        }
    }

    /// Replace the instruction cycles function.
    fn with_cycles<F>(self, cycles: F) -> WithCyclesFunc<Self, F>
    where
        Self: Sized,
        F: Fn(Instruction) -> u64,
    {
        WithCyclesFunc { base: self, cycles }
    }
}

impl<Mac: SupportMachine, T: ExecutionContext<Mac>> ExecutionContextExt<Mac> for T {}
