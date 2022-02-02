use crate::priv_prelude::*;

macro_rules! define_opcode (
    ($ty_name:ident, $fn_name:ident, $s:literal) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        pub fn $fn_name() -> impl Parser<Output = $ty_name> + Clone {
            keyword($s).map_with_span(|(), span| $ty_name { span })
        }
    );
);

define_opcode!(AddOpcode, add_opcode, "add");
define_opcode!(AddiOpcode, addi_opcode, "addi");
define_opcode!(AndOpcode, and_opcode, "and");
define_opcode!(AndiOpcode, andi_opcode, "andi");
define_opcode!(DivOpcode, div_opcode, "div");
define_opcode!(DiviOpcode, divi_opcode, "divi");
define_opcode!(EqOpcode, eq_opcode, "eq");
define_opcode!(ExpOpcode, exp_opcode, "exp");
define_opcode!(ExpiOpcode, expi_opcode, "expi");
define_opcode!(GtOpcode, gt_opcode, "gt");
define_opcode!(LtOpcode, lt_opcode, "lt");
define_opcode!(MlogOpcode, mlog_opcode, "mlog");
define_opcode!(ModOpcode, mod_opcode, "mod");
define_opcode!(ModiOpcode, modi_opcode, "modi");
define_opcode!(MoveOpcode, move_opcode, "move");
define_opcode!(MrooOpcode, mroo_opcode, "mroo");
define_opcode!(MulOpcode, mul_opcode, "mul");
define_opcode!(MuliOpcode, muli_opcode, "muli");
define_opcode!(NoopOpcode, noop_opcode, "noop");
define_opcode!(NotOpcode, not_opcode, "not");
define_opcode!(OrOpcode, or_opcode, "or");
define_opcode!(OriOpcode, ori_opcode, "ori");
define_opcode!(SllOpcode, sll_opcode, "sll");
define_opcode!(SlliOpcode, slli_opcode, "slli");
define_opcode!(SrlOpcode, srl_opcode, "srl");
define_opcode!(SrliOpcode, srli_opcode, "srli");
define_opcode!(SubOpcode, sub_opcode, "sub");
define_opcode!(SubiOpcode, subi_opcode, "subi");
define_opcode!(XorOpcode, xor_opcode, "xor");
define_opcode!(XoriOpcode, xori_opcode, "xori");
define_opcode!(CimvOpcode, cimv_opcode, "cimv");
define_opcode!(CtmvOpcode, ctmv_opcode, "ctmv");
define_opcode!(JiOpcode, ji_opcode, "ji");
define_opcode!(JneiOpcode, jnei_opcode, "jnei");
define_opcode!(RetOpcode, ret_opcode, "ret");
define_opcode!(AlocOpcode, aloc_opcode, "aloc");
define_opcode!(CfeiOpcode, cfei_opcode, "cfei");
define_opcode!(CfsiOpcode, cfsi_opcode, "cfsi");
define_opcode!(LbOpcode, lb_opcode, "lb");
define_opcode!(LwOpcode, lw_opcode, "lw");
define_opcode!(MclOpcode, mcl_opcode, "mcl");
define_opcode!(McliOpcode, mcli_opcode, "mcli");
define_opcode!(McpOpcode, mcp_opcode, "mcp");
define_opcode!(McpiOpcode, mcpi_opcode, "mcpi");
define_opcode!(MeqOpcode, meq_opcode, "meq");
define_opcode!(SbOpcode, sb_opcode, "sb");
define_opcode!(SwOpcode, sw_opcode, "sw");
define_opcode!(BalOpcode, bal_opcode, "bal");
define_opcode!(BheiOpcode, bhei_opcode, "bhei");
define_opcode!(BhshOpcode, bhsh_opcode, "bhsh");
define_opcode!(BurnOpcode, burn_opcode, "burn");
define_opcode!(CallOpcode, call_opcode, "call");
define_opcode!(CbOpcode, cb_opcode, "cb");
define_opcode!(CcpOpcode, ccp_opcode, "ccp");
define_opcode!(CrooOpcode, croo_opcode, "croo");
define_opcode!(CsizOpcode, csiz_opcode, "csiz");
define_opcode!(LdcOpcode, ldc_opcode, "ldc");
define_opcode!(LogOpcode, log_opcode, "log");
define_opcode!(LogdOpcode, logd_opcode, "logd");
define_opcode!(MintOpcode, mint_opcode, "mint");
define_opcode!(RetdOpcode, retd_opcode, "retd");
define_opcode!(RvrtOpcode, rvrt_opcode, "rvrt");
define_opcode!(SldcOpcode, sldc_opcode, "sldc");
define_opcode!(SrwOpcode, srw_opcode, "srw");
define_opcode!(SrwqOpcode, srwq_opcode, "srwq");
define_opcode!(SwwOpcode, sww_opcode, "sww");
define_opcode!(SwwqOpcode, swwq_opcode, "swwq");
define_opcode!(TrOpcode, tr_opcode, "tr");
define_opcode!(TroOpcode, tro_opcode, "tro");
define_opcode!(EcrOpcode, ecr_opcode, "ecr");
define_opcode!(K256Opcode, k256_opcode, "k256");
define_opcode!(S256Opcode, s256_opcode, "s256");
define_opcode!(XilOpcode, xil_opcode, "xil");
define_opcode!(XisOpcode, xis_opcode, "xis");
define_opcode!(XolOpcode, xol_opcode, "xol");
define_opcode!(XosOpcode, xos_opcode, "xos");
define_opcode!(XwlOpcode, xwl_opcode, "xwl");
define_opcode!(XwsOpcode, xws_opcode, "xws");
define_opcode!(FlagOpcode, flag_opcode, "flag");
define_opcode!(GmOpcode, gm_opcode, "gm");

