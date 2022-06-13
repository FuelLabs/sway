use crate::{
    asm_generation::{register_allocator, DataSection, InstructionSet, RegisterSequencer},
    asm_lang::{
        allocated_ops::AllocatedOp, Label, Op, OrganizationalOp, RealizedOp, VirtualImmediate12,
        VirtualImmediate18, VirtualImmediate24, VirtualOp,
    },
};
use std::{collections::HashMap, fmt};

use either::Either;

/// An [AbstractInstructionSet] is a set of instructions that use entirely virtual registers
/// and excessive moves, with the intention of later optimizing it.
#[derive(Clone)]
pub struct AbstractInstructionSet {
    pub(crate) ops: Vec<Op>,
}

impl AbstractInstructionSet {
    /// Removes any jumps that jump to the subsequent line
    pub(crate) fn remove_sequential_jumps(&self) -> AbstractInstructionSet {
        let mut buf = vec![];
        for i in 0..self.ops.len() - 1 {
            if let Op {
                opcode: Either::Right(OrganizationalOp::Jump(ref label)),
                ..
            } = self.ops[i]
            {
                if let Op {
                    opcode: Either::Right(OrganizationalOp::Label(ref label2)),
                    ..
                } = self.ops[i + 1]
                {
                    if label == label2 {
                        // this is a jump to the next line
                        // omit these by doing nothing
                        continue;
                    }
                }
            }
            buf.push(self.ops[i].clone());
        }
        // the last item cannot sequentially jump by definition so we add it in here
        if let Some(x) = self.ops.last() {
            buf.push(x.clone())
        };

        // scan through the jumps and remove any labels that are unused
        // this could of course be N instead of 2N if i did this in the above for loop.
        // However, the sweep for unused labels is inevitable regardless of the above phase
        // so might as well do it here.
        let mut buf2 = vec![];
        for op in &buf {
            match op.opcode {
                Either::Right(OrganizationalOp::Label(ref label)) => {
                    if label_is_used(&buf, label) {
                        buf2.push(op.clone());
                    }
                }
                _ => buf2.push(op.clone()),
            }
        }

        AbstractInstructionSet { ops: buf2 }
    }

    /// Runs two passes -- one to get the instruction offsets of the labels
    /// and one to replace the labels in the organizational ops
    pub(crate) fn realize_labels(
        self,
        data_section: &DataSection,
    ) -> RealizedAbstractInstructionSet {
        let mut label_namespace: HashMap<&Label, u64> = Default::default();
        let mut offset_map = vec![];
        let mut counter = 0;
        for op in &self.ops {
            offset_map.push(counter);
            match op.opcode {
                Either::Right(OrganizationalOp::Label(ref lab)) => {
                    label_namespace.insert(lab, counter);
                }
                // A special case for LWDataId which may be 1 or 2 ops, depending on the source size.
                Either::Left(VirtualOp::LWDataId(_, ref data_id)) => {
                    let type_of_data = data_section.type_of_data(data_id).expect(
                        "Internal miscalculation in data section -- data id did not match up to any actual data",
                    );
                    counter += if type_of_data.is_copy_type() { 1 } else { 2 };
                }
                // these ops will end up being exactly one op, so the counter goes up one
                Either::Right(OrganizationalOp::Jump(..))
                | Either::Right(OrganizationalOp::JumpIfNotEq(..))
                | Either::Right(OrganizationalOp::JumpIfNotZero(..))
                | Either::Left(_) => {
                    counter += 1;
                }
                Either::Right(OrganizationalOp::Comment) => (),
                Either::Right(OrganizationalOp::DataSectionOffsetPlaceholder) => {
                    // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                    // to load the data, which loads a whole word, so for now this is 2.
                    counter += 2
                }
            }
        }

        let mut realized_ops = vec![];
        for (
            ix,
            Op {
                opcode,
                owning_span,
                comment,
            },
        ) in self.ops.clone().into_iter().enumerate()
        {
            let offset = offset_map[ix];
            match opcode {
                Either::Left(op) => realized_ops.push(RealizedOp {
                    opcode: op,
                    owning_span,
                    comment,
                    offset,
                }),
                Either::Right(org_op) => match org_op {
                    OrganizationalOp::Jump(ref lab) => {
                        let imm = VirtualImmediate24::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^24 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::JI(imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    OrganizationalOp::JumpIfNotEq(r1, r2, ref lab) => {
                        let imm = VirtualImmediate12::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^12 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::JNEI(r1, r2, imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    OrganizationalOp::JumpIfNotZero(r1, ref lab) => {
                        let imm = VirtualImmediate18::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^18 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::JNZI(r1, imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    OrganizationalOp::DataSectionOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                            offset,
                        });
                    }
                    OrganizationalOp::Comment => continue,
                    OrganizationalOp::Label(..) => continue,
                },
            };
        }
        RealizedAbstractInstructionSet { ops: realized_ops }
    }
}

impl fmt::Display for AbstractInstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// "Realized" here refers to labels -- there are no more organizational
/// ops or labels. In this struct, they are all "realized" to offsets.
pub struct RealizedAbstractInstructionSet {
    ops: Vec<RealizedOp>,
}

impl RealizedAbstractInstructionSet {
    /// Assigns an allocatable register to each virtual register used by some instruction in the
    /// list `self.ops`. The algorithm used is Chaitin's graph-coloring register allocation
    /// algorithm (https://en.wikipedia.org/wiki/Chaitin%27s_algorithm). The individual steps of
    /// the algorithm are thoroughly explained in register_allocator.rs.
    ///
    pub(crate) fn allocate_registers(
        self,
        register_sequencer: &mut RegisterSequencer,
    ) -> InstructionSet {
        // Step 1: Liveness Analysis.
        let live_out = register_allocator::liveness_analysis(&self.ops);

        // Step 2: Construct the interference graph.
        let (mut interference_graph, mut reg_to_node_ix) =
            register_allocator::create_interference_graph(&self.ops, &live_out);

        // Step 3: Remove redundant MOVE instructions using the interference graph.
        let reduced_ops = register_allocator::coalesce_registers(
            &self.ops,
            &mut interference_graph,
            &mut reg_to_node_ix,
            register_sequencer,
        );

        // Step 4: Simplify - i.e. color the interference graph and return a stack that contains
        // each colorable node and its neighbors.
        let mut stack = register_allocator::color_interference_graph(&mut interference_graph);

        // Step 5: Use the stack to assign a register for each virtual register.
        let pool = register_allocator::assign_registers(&mut stack);

        // Steph 6: Update all instructions to use the resulting register pool.
        let mut buf = vec![];
        for op in &reduced_ops {
            buf.push(AllocatedOp {
                opcode: op.opcode.allocate_registers(&pool),
                comment: op.comment.clone(),
                owning_span: op.owning_span.clone(),
            })
        }

        InstructionSet { ops: buf }
    }
}

/// helper function to check if a label is used in a given buffer of ops
fn label_is_used(buf: &[Op], label: &Label) -> bool {
    buf.iter().any(|Op { ref opcode, .. }| match opcode {
        Either::Right(OrganizationalOp::Jump(ref l)) if label == l => true,
        Either::Right(OrganizationalOp::JumpIfNotEq(_, _, ref l)) if label == l => true,
        Either::Right(OrganizationalOp::JumpIfNotZero(_, ref l)) if label == l => true,
        _ => false,
    })
}
