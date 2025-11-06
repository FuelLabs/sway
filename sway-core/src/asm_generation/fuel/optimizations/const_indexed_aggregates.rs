use rustc_hash::FxHashMap;

use crate::{
    asm_generation::fuel::compiler_constants,
    asm_lang::{VirtualImmediate12, VirtualOp, VirtualRegister},
};

use super::super::{abstract_instruction_set::AbstractInstructionSet, data_section::DataSection};

impl AbstractInstructionSet {
    // Aggregates that are const index accessed from a base address
    // can use the IMM field of LW/SW if the value fits in 12 bits.
    // Only the LW/SW instructions are modified, and the redundant
    // computations left untouched, to be later removed by a DCE pass.
    pub(crate) fn const_indexing_aggregates_function(mut self, data_section: &DataSection) -> Self {
        // Poor man's SSA (local ... per block).
        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        struct VRegDef {
            reg: VirtualRegister,
            ver: u32,
        }

        // What does a register contain?
        #[derive(Debug, PartialEq, Eq)]
        enum RegContents {
            Constant(u64),
            BaseOffset(VRegDef, u64),
        }

        // What is the latest version of a vreg definition.
        let mut latest_version = FxHashMap::<VirtualRegister, u32>::default();
        // Track register contents as we progress instructions in a block.
        let mut reg_contents = FxHashMap::<VirtualRegister, RegContents>::default();

        // Record that we saw a new definition of `reg`.
        fn record_new_def(
            latest_version: &mut FxHashMap<VirtualRegister, u32>,
            reg: &VirtualRegister,
        ) {
            latest_version
                .entry(reg.clone())
                .and_modify(|ver| *ver += 1)
                .or_insert(1);
        }

        // What's the latest definition we've seen of `reg`?
        fn get_def_version(
            latest_version: &FxHashMap<VirtualRegister, u32>,
            reg: &VirtualRegister,
        ) -> u32 {
            latest_version.get(reg).cloned().unwrap_or(0)
        }

        fn process_add(
            reg_contents: &mut FxHashMap<VirtualRegister, RegContents>,
            latest_version: &mut FxHashMap<VirtualRegister, u32>,
            dest: &VirtualRegister,
            opd1: &VirtualRegister,
            c2: u64,
        ) {
            match reg_contents.get(opd1) {
                Some(RegContents::Constant(c1)) if c1.checked_add(c2).is_some() => {
                    reg_contents.insert(dest.clone(), RegContents::Constant(c1 + c2));
                    record_new_def(latest_version, dest);
                }
                Some(RegContents::BaseOffset(base_reg, offset))
                    if get_def_version(latest_version, &base_reg.reg) == base_reg.ver
                        && offset.checked_add(c2).is_some() =>
                {
                    reg_contents.insert(
                        dest.clone(),
                        RegContents::BaseOffset(base_reg.clone(), offset + c2),
                    );
                    record_new_def(latest_version, dest);
                }
                _ => {
                    let base = VRegDef {
                        reg: opd1.clone(),
                        ver: get_def_version(latest_version, opd1),
                    };
                    reg_contents.insert(dest.clone(), RegContents::BaseOffset(base, c2));
                    record_new_def(latest_version, dest);
                }
            }
        }

        self.ops.retain_mut(|op| {
            let mut retain = true;
            let mut clear_state = false;

            // Uncomment to debug what this optimization is doing
            // let op_before = op.clone();

            match &mut op.opcode {
                either::Either::Left(op) => {
                    match op {
                        VirtualOp::ADD(dest, opd1, opd2) => {
                            // We don't look for the first operand being a constant and the second
                            // one a base register. Such patterns must be canonicalised prior.
                            if let Some(&RegContents::Constant(c2)) = reg_contents.get(opd2) {
                                process_add(&mut reg_contents, &mut latest_version, dest, opd1, c2);
                            } else {
                                reg_contents.remove(dest);
                                record_new_def(&mut latest_version, dest);
                            };
                        }
                        VirtualOp::ADDI(dest, opd1, opd2) => {
                            let c2 = opd2.value() as u64;
                            process_add(&mut reg_contents, &mut latest_version, dest, opd1, c2);
                        }
                        VirtualOp::MUL(dest, opd1, opd2) => {
                            match (reg_contents.get(opd1), reg_contents.get(opd2)) {
                                (
                                    Some(RegContents::Constant(c1)),
                                    Some(RegContents::Constant(c2)),
                                ) => {
                                    reg_contents
                                        .insert(dest.clone(), RegContents::Constant(c1 * c2));
                                    record_new_def(&mut latest_version, dest);
                                }
                                _ => {
                                    reg_contents.remove(dest);
                                    record_new_def(&mut latest_version, dest);
                                }
                            }
                        }
                        VirtualOp::LoadDataId(dest, data_id) => {
                            if let Some(c) = data_section.get_data_word(data_id) {
                                reg_contents.insert(dest.clone(), RegContents::Constant(c));
                            } else {
                                reg_contents.remove(dest);
                            }
                            record_new_def(&mut latest_version, dest);
                        }
                        VirtualOp::MOVI(dest, imm) => {
                            reg_contents
                                .insert(dest.clone(), RegContents::Constant(imm.value() as u64));
                            record_new_def(&mut latest_version, dest);
                        }
                        VirtualOp::LW(dest, addr_reg, imm) => match reg_contents.get(addr_reg) {
                            Some(RegContents::BaseOffset(base_reg, offset))
                                if offset % 8 == 0
                                    && get_def_version(&latest_version, &base_reg.reg)
                                        == base_reg.ver
                                    && ((offset / 8) + imm.value() as u64)
                                        < compiler_constants::TWELVE_BITS =>
                            {
                                let new_imm =
                                    VirtualImmediate12::new((offset / 8) + imm.value() as u64);
                                let new_lw =
                                    VirtualOp::LW(dest.clone(), base_reg.reg.clone(), new_imm);
                                // The register defined is no more useful for us. Forget anything from its past.
                                reg_contents.remove(dest);
                                record_new_def(&mut latest_version, dest);
                                // Replace the LW with a new one in-place.
                                *op = new_lw;
                            }
                            _ => {
                                reg_contents.remove(dest);
                                record_new_def(&mut latest_version, dest);
                            }
                        },
                        VirtualOp::SW(addr_reg, src, imm) => match reg_contents.get(addr_reg) {
                            Some(RegContents::BaseOffset(base_reg, offset))
                                if offset % 8 == 0
                                    && get_def_version(&latest_version, &base_reg.reg)
                                        == base_reg.ver
                                    && ((offset / 8) + imm.value() as u64)
                                        < compiler_constants::TWELVE_BITS =>
                            {
                                let new_imm =
                                    VirtualImmediate12::new((offset / 8) + imm.value() as u64);
                                let new_sw =
                                    VirtualOp::SW(base_reg.reg.clone(), src.clone(), new_imm);
                                // Replace the SW with a new one in-place.
                                *op = new_sw;
                            }
                            _ => (),
                        },
                        VirtualOp::MOVE(dest, src) => {
                            let ver = get_def_version(&latest_version, src);
                            if let Some(RegContents::BaseOffset(src, 0)) = reg_contents.get(src) {
                                if dest == &src.reg && src.ver == ver {
                                    retain = false;
                                }
                            } else {
                                reg_contents.insert(
                                    dest.clone(),
                                    RegContents::BaseOffset(
                                        VRegDef {
                                            reg: src.clone(),
                                            ver,
                                        },
                                        0,
                                    ),
                                );
                            }
                        }
                        _ => {
                            // For every Op that we don't know about,
                            // forget everything we know about its def registers.
                            for def_reg in op.def_registers() {
                                reg_contents.remove(def_reg);
                                record_new_def(&mut latest_version, def_reg);
                            }
                        }
                    }
                }
                either::Either::Right(_) => {
                    clear_state = true;
                }
            }

            // Uncomment to debug what this optimization is doing
            //let before = op_before.opcode.to_string();
            //let after = op.opcode.to_string();
            // println!("{}", before);

            if clear_state {
                latest_version.clear();
                reg_contents.clear();
                // println!("    state cleared");
            }

            // Uncomment to debug what this optimization is doing
            // if before != after {
            //     println!("    optimized to");
            //     println!("    {}", after);
            //     println!("    using");
            //     for (k, v) in reg_contents.iter() {
            //         println!("    - {:?} -> {:?}", k, v);
            //     }
            // }
            // if !retain {
            //     println!("    removed");
            //     for (k, v) in reg_contents.iter() {
            //         println!("    - {:?} -> {:?}", k, v);
            //     }
            // }
            // if forget_def_registers {
            //     for def_reg in op.def_registers() {
            //         println!("    forget {}", def_reg.to_string());
            //     }
            // }

            retain
        });

        self
    }
}
