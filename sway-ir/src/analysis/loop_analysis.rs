use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, Block, Context, DebugWithContext, DomTree,
    Function, IrError, Pass, PassMutability, PostOrder, ScopedPass, DOMINATORS_NAME,
    POSTORDER_NAME,
};

pub const LOOP_ANALYSIS_NAME: &str = "loop analysis";

pub fn create_loop_analysis_pass() -> Pass {
    Pass {
        name: LOOP_ANALYSIS_NAME,
        descr: "Loop analysis computation",
        deps: vec![POSTORDER_NAME, DOMINATORS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_loop_analysis_pass)),
    }
}

#[derive(Debug)]
pub struct Loop {
    blocks: HashSet<Block>,
}

pub struct LoopAnalysis {
    loops: Vec<Loop>,
    block_to_loops: HashMap<Block, Vec<usize>>,
}

impl LoopAnalysis {
    pub fn is_inside_loop(&self, block: &Block) -> bool {
        self.block_to_loops
            .get(block)
            .map(|b| !b.is_empty())
            .unwrap_or_default()
    }
}

impl DebugWithContext for LoopAnalysis {
    fn fmt_with_context(&self, f: &mut std::fmt::Formatter, context: &Context) -> std::fmt::Result {
        let mut block_to_loops = BTreeMap::new();

        let mut keys = self.block_to_loops.keys().collect::<Vec<_>>();
        keys.sort_by_key(|b| b.0);
        for block in keys {
            let loops = self.block_to_loops.get(block).expect("key not found");
            block_to_loops.insert(block.get_label(context), loops);
        }

        let mut loops = BTreeMap::new();
        for (idx, l) in self.loops.iter().enumerate() {
            let mut blocks = l
                .blocks
                .iter()
                .map(|x| x.get_label(context))
                .collect::<Vec<_>>();
            blocks.sort();
            loops.insert(idx, blocks);
        }

        f.debug_struct("LoopAnalysis")
            .field("block_to_loops", &block_to_loops)
            .field("loops", &loops)
            .finish()
    }
}

impl AnalysisResultT for LoopAnalysis {}

/// Compute if instructions are inside of loops or not
fn compute_loop_analysis_pass(
    context: &Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    let dom_tree: &DomTree = analyses.get_analysis_result(function);
    let po: &PostOrder = analyses.get_analysis_result(function);

    let result = compute_loop_analysis(context, dom_tree, po)?;

    Ok(Box::new(result))
}

fn compute_loop_analysis(
    context: &Context<'_>,
    dom_tree: &DomTree,
    po: &PostOrder,
) -> Result<LoopAnalysis, IrError> {
    // 2.  **Find all Back-edges**:
    // Iterate through every edge $(U, V)$ in the CFG.
    // If $V$ is an ancestor of $U$ in the Dominator Tree, $(U, V)$ is a back-edge.
    let mut back_edges = vec![];

    for block in po.po_to_block.iter() {
        for branch in block.successors(context) {
            let successor = branch.block;
            if dom_tree.dominates(successor, *block) {
                back_edges.push((*block, successor));
            }
        }
    }

    let mut block_to_loops = HashMap::<Block, Vec<usize>>::new();

    // 3.  **Map Blocks to Loops**
    // For every back-edge $(U, V)$, find all blocks that can reach $U$ without passing through $V$.
    // Mark these blocks as "part of a loop."
    let mut loops = vec![];

    for (loop_tail, loop_header) in back_edges {
        let loop_id = loops.len();

        let mut loop_blocks = HashSet::new();
        let mut q = vec![loop_tail];
        while let Some(block) = q.pop() {
            if !loop_blocks.insert(block) {
                continue;
            }

            // Map blocks to loops
            block_to_loops.entry(block).or_default().push(loop_id);

            if block == loop_header {
                continue;
            }

            let reachabe_preds = block
                .pred_iter(context)
                .filter(|block| po.is_reachable(block));
            q.extend(reachabe_preds);
        }

        assert!(loop_blocks.contains(&loop_tail));
        assert!(loop_blocks.contains(&loop_header));

        loops.push(Loop {
            blocks: loop_blocks,
        });
    }

    Ok(LoopAnalysis {
        loops,
        block_to_loops,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        compute_dom_tree, compute_post_order, loop_analysis::compute_loop_analysis, Backtrace,
        DebugWithContext,
    };
    use sway_features::ExperimentalFeatures;
    use sway_types::SourceEngine;

    fn parse<'a>(se: &'a SourceEngine, body: &str) -> crate::Context<'a> {
        let context = crate::parse(
            &format!(
                "script {{
                {body}
            }}

            !0 = \"a.sw\"
            "
            ),
            se,
            ExperimentalFeatures::default(),
            Backtrace::default(),
        )
        .unwrap();
        context
    }

    #[test]
    fn must_id_instructions_on_while_loops() {
        let se = SourceEngine::default();
        // #[inline(never)]
        // fn simple_while() {
        //     let mut counter = 0;
        //     while counter < 10 {
        //         counter = counter + 1;
        //     }
        // }
        let ir = parse(
            &se,
            "fn simple_while_1() -> () {
local mut u64 counter
local u64 other_
local u64 other_0
local u64 other_1

entry():
v169v1 = get_local __ptr u64, counter
v170v1 = const u64 0
store v170v1 to v169v1
br while()

while():
v173v1 = get_local __ptr u64, counter
v174v1 = get_local __ptr u64, other_
v175v1 = const u64 10
store v175v1 to v174v1
v177v1 = load v173v1
v178v1 = get_local __ptr u64, other_
v179v1 = load v178v1
v180v1 = cmp lt v177v1 v179v1
cbr v180v1, while_body(), end_while()

while_body():
v182v1 = get_local __ptr u64, counter
v183v1 = get_local __ptr u64, other_0
v184v1 = const u64 1
store v184v1 to v183v1
v186v1 = load v182v1
v187v1 = get_local __ptr u64, other_0
v188v1 = load v187v1
v189v1 = add v186v1, v188v1
v190v1 = get_local __ptr u64, counter
store v189v1 to v190v1
br while()

end_while():
v193v1 = get_local __ptr u64, counter
v194v1 = get_local __ptr u64, other_1
v195v1 = const u64 10
store v195v1 to v194v1
v197v1 = load v193v1
v198v1 = get_local __ptr u64, other_1
v199v1 = load v198v1
v200v1 = cmp eq v197v1 v199v1
v202v1 = const unit ()
ret () v202v1
}",
        );

        let function = ir
            .module_iter()
            .find_map(|m| {
                m.function_iter(&ir)
                    .find(|f| f.get_name(&ir) == "simple_while_1")
            })
            .unwrap();

        let po = compute_post_order(&ir, &function);
        let domtree = compute_dom_tree(&ir, function, &po).unwrap();
        let r = compute_loop_analysis(&ir, &domtree, &po).unwrap();
        expect_test::expect![[r#"
            LoopAnalysis {
                block_to_loops: {
                    "while": [
                        0,
                    ],
                    "while_body": [
                        0,
                    ],
                },
                loops: {
                    0: [
                        "while",
                        "while_body",
                    ],
                },
            }"#]]
        .assert_eq(&format!("{:#?}", r.with_context(&ir)));
    }

    #[test]
    fn must_id_instructions_on_for_loops() {
        let se = SourceEngine::default();
        // #[inline(never)]
        // fn just_for_loop(vector: Vec<u64>) {
        //     let mut i = 0;
        //     for n in vector.iter() {
        //         i += 1;
        //     }
        // }
        let ir = parse(
            &se,
            "fn just_for_loop_15(vector: __ptr { { ptr, u64 }, u64 }) -> () {
local mut { { { ptr, u64 }, u64 }, u64 } __for_iterable_2
local mut { u64, ( () | u64 ) } __for_value_opt_1
local { u64, ( () | u64 ) } __ret_val
local { { { ptr, u64 }, u64 }, u64 } __struct_init_0
local mut u64 i
local u64 n
local u64 other_
local u64 other_0
local u64 other_1

entry(vector: __ptr { { ptr, u64 }, u64 }):
v1132v1 = get_local __ptr u64, i
v1133v1 = const u64 0
store v1133v1 to v1132v1
v1135v1 = get_local __ptr { { { ptr, u64 }, u64 }, u64 }, __struct_init_0
v1136v1 = const u64 0
v1137v1 = get_elem_ptr v1135v1, __ptr { { ptr, u64 }, u64 }, v1136v1
mem_copy_val v1137v1, vector
v1139v1 = const u64 1
v1140v1 = get_elem_ptr v1135v1, __ptr u64, v1139v1
v1141v1 = const u64 0
store v1141v1 to v1140v1
v1143v1 = get_local __ptr { { { ptr, u64 }, u64 }, u64 }, __for_iterable_2
mem_copy_val v1143v1, v1135v1
br while()

while():
v1146v1 = const bool true
cbr v1146v1, while_body(), end_while()

while_body():
v1148v1 = get_local __ptr { { { ptr, u64 }, u64 }, u64 }, __for_iterable_2
v1149v1 = get_local __ptr { u64, ( () | u64 ) }, __ret_val
v1151v1 = get_local __ptr { u64, ( () | u64 ) }, __for_value_opt_1
mem_copy_val v1151v1, v1149v1
v1153v1 = get_local __ptr { u64, ( () | u64 ) }, __for_value_opt_1
v1154v1 = const u64 0
v1155v1 = get_elem_ptr v1153v1, __ptr u64, v1154v1
v1156v1 = get_local __ptr u64, other_
v1157v1 = const u64 1
store v1157v1 to v1156v1
v1159v1 = load v1155v1
v1160v1 = get_local __ptr u64, other_
v1161v1 = load v1160v1
v1162v1 = cmp eq v1159v1 v1161v1
v1163v1 = const bool false
cbr v1162v1, is_none_22_block2(v1163v1), is_none_22_block1()

is_none_22_block1():
v1165v1 = const bool true
br is_none_22_block2(v1165v1)

is_none_22_block2(v1131v1: bool):
cbr v1131v1, end_while(), block1()

block1():
v1168v1 = get_local __ptr { u64, ( () | u64 ) }, __for_value_opt_1
v1170v1 = get_local __ptr u64, n
v1172v1 = get_local __ptr u64, i
v1173v1 = get_local __ptr u64, n
v1174v1 = load v1173v1
v1175v1 = load v1172v1
v1176v1 = cmp eq v1174v1 v1175v1
v1178v1 = get_local __ptr u64, i
v1179v1 = get_local __ptr u64, other_0
v1180v1 = const u64 1
store v1180v1 to v1179v1
v1182v1 = load v1178v1
v1183v1 = get_local __ptr u64, other_0
v1184v1 = load v1183v1
v1185v1 = add v1182v1, v1184v1
v1186v1 = get_local __ptr u64, i
store v1185v1 to v1186v1
br while()

end_while():
v1189v1 = get_local __ptr u64, i
v1190v1 = get_local __ptr u64, other_1
v1191v1 = const u64 5
store v1191v1 to v1190v1
v1193v1 = load v1189v1
v1194v1 = get_local __ptr u64, other_1
v1195v1 = load v1194v1
v1196v1 = cmp eq v1193v1 v1195v1
v1198v1 = const unit ()
ret () v1198v1
}",
        );

        let function = ir
            .module_iter()
            .find_map(|m| {
                m.function_iter(&ir)
                    .find(|f| f.get_name(&ir) == "just_for_loop_15")
            })
            .unwrap();

        let po = compute_post_order(&ir, &function);
        let domtree = compute_dom_tree(&ir, function, &po).unwrap();
        let r = compute_loop_analysis(&ir, &domtree, &po).unwrap();
        expect_test::expect![[r#"
            LoopAnalysis {
                block_to_loops: {
                    "block1": [
                        0,
                    ],
                    "is_none_22_block1": [
                        0,
                    ],
                    "is_none_22_block2": [
                        0,
                    ],
                    "while": [
                        0,
                    ],
                    "while_body": [
                        0,
                    ],
                },
                loops: {
                    0: [
                        "block1",
                        "is_none_22_block1",
                        "is_none_22_block2",
                        "while",
                        "while_body",
                    ],
                },
            }"#]]
        .assert_eq(&format!("{:#?}", r.with_context(&ir)));
    }
}
