use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct AsmInstruction {
    pub opcode: Opcode,
    pub args: Vec<AsmArg>,
    pub semicolon_token: SemicolonToken,
}

#[derive(Clone, Debug)]
pub enum Opcode {
    Add(AddOpcode),
    Addi(AddiOpcode),
    And(AndOpcode),
    Andi(AndiOpcode),
    Div(DivOpcode),
    Divi(DiviOpcode),
    Eq(EqOpcode),
    Exp(ExpOpcode),
    Expi(ExpiOpcode),
    Gt(GtOpcode),
    Lt(LtOpcode),
    Mlog(MlogOpcode),
    Mod(ModOpcode),
    Modi(ModiOpcode),
    Move(MoveOpcode),
    Mroo(MrooOpcode),
    Mul(MulOpcode),
    Muli(MuliOpcode),
    Noop(NoopOpcode),
    Not(NotOpcode),
    Or(OrOpcode),
    Ori(OriOpcode),
    Sll(SllOpcode),
    Slli(SlliOpcode),
    Srl(SrlOpcode),
    Srli(SrliOpcode),
    Sub(SubOpcode),
    Subi(SubiOpcode),
    Xor(XorOpcode),
    Xori(XoriOpcode),
    Cimv(CimvOpcode),
    Ctmv(CtmvOpcode),
    Ji(JiOpcode),
    Jnei(JneiOpcode),
    Ret(RetOpcode),
    Aloc(AlocOpcode),
    Cfei(CfeiOpcode),
    Cfsi(CfsiOpcode),
    Lb(LbOpcode),
    Lw(LwOpcode),
    Mcl(MclOpcode),
    Mcli(McliOpcode),
    Mcp(McpOpcode),
    Mcpi(McpiOpcode),
    Meq(MeqOpcode),
    Sb(SbOpcode),
    Sw(SwOpcode),
    Bal(BalOpcode),
    Bhei(BheiOpcode),
    Bhsh(BhshOpcode),
    Burn(BurnOpcode),
    Call(CallOpcode),
    Cb(CbOpcode),
    Ccp(CcpOpcode),
    Croo(CrooOpcode),
    Csiz(CsizOpcode),
    Ldc(LdcOpcode),
    Log(LogOpcode),
    Logd(LogdOpcode),
    Mint(MintOpcode),
    Retd(RetdOpcode),
    Rvrt(RvrtOpcode),
    Sldc(SldcOpcode),
    Srw(SrwOpcode),
    Srwq(SrwqOpcode),
    Sww(SwwOpcode),
    Swwq(SwwqOpcode),
    Tr(TrOpcode),
    Tro(TroOpcode),
    Ecr(EcrOpcode),
    K256(K256Opcode),
    S256(S256Opcode),
    Xil(XilOpcode),
    Xis(XisOpcode),
    Xol(XolOpcode),
    Xos(XosOpcode),
    Xwl(XwlOpcode),
    Xws(XwsOpcode),
    Flag(FlagOpcode),
    Gm(GmOpcode),
}

#[derive(Clone, Debug)]
pub enum AsmArg {
    Register(Ident),
    Immediate(AsmImmediate),
}

pub fn asm_instruction() -> impl Parser<Output = AsmInstruction> + Clone {
    opcode()
    .then(leading_whitespace(asm_arg()).repeated())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|((opcode, args), semicolon_token)| {
        AsmInstruction { opcode, args, semicolon_token }
    })
}

pub fn opcode() -> impl Parser<Output = Opcode> + Clone {
    let op_add = add_opcode().map(Opcode::Add);
    let op_addi = addi_opcode().map(Opcode::Addi);
    let op_and = and_opcode().map(Opcode::And);
    let op_andi = andi_opcode().map(Opcode::Andi);
    let op_div = div_opcode().map(Opcode::Div);
    let op_divi = divi_opcode().map(Opcode::Divi);
    let op_eq = eq_opcode().map(Opcode::Eq);
    let op_exp = exp_opcode().map(Opcode::Exp);
    let op_expi = expi_opcode().map(Opcode::Expi);
    let op_gt = gt_opcode().map(Opcode::Gt);
    let op_lt = lt_opcode().map(Opcode::Lt);
    let op_mlog = mlog_opcode().map(Opcode::Mlog);
    let op_mod = mod_opcode().map(Opcode::Mod);
    let op_modi = modi_opcode().map(Opcode::Modi);
    let op_move = move_opcode().map(Opcode::Move);
    let op_mroo = mroo_opcode().map(Opcode::Mroo);
    let op_mul = mul_opcode().map(Opcode::Mul);
    let op_muli = muli_opcode().map(Opcode::Muli);
    let op_noop = noop_opcode().map(Opcode::Noop);
    let op_not = not_opcode().map(Opcode::Not);
    let op_or = or_opcode().map(Opcode::Or);
    let op_ori = ori_opcode().map(Opcode::Ori);
    let op_sll = sll_opcode().map(Opcode::Sll);
    let op_slli = slli_opcode().map(Opcode::Slli);
    let op_srl = srl_opcode().map(Opcode::Srl);
    let op_srli = srli_opcode().map(Opcode::Srli);
    let op_sub = sub_opcode().map(Opcode::Sub);
    let op_subi = subi_opcode().map(Opcode::Subi);
    let op_xor = xor_opcode().map(Opcode::Xor);
    let op_xori = xori_opcode().map(Opcode::Xori);
    let op_cimv = cimv_opcode().map(Opcode::Cimv);
    let op_ctmv = ctmv_opcode().map(Opcode::Ctmv);
    let op_ji = ji_opcode().map(Opcode::Ji);
    let op_jnei = jnei_opcode().map(Opcode::Jnei);
    let op_ret = ret_opcode().map(Opcode::Ret);
    let op_aloc = aloc_opcode().map(Opcode::Aloc);
    let op_cfei = cfei_opcode().map(Opcode::Cfei);
    let op_cfsi = cfsi_opcode().map(Opcode::Cfsi);
    let op_lb = lb_opcode().map(Opcode::Lb);
    let op_lw = lw_opcode().map(Opcode::Lw);
    let op_mcl = mcl_opcode().map(Opcode::Mcl);
    let op_mcli = mcli_opcode().map(Opcode::Mcli);
    let op_mcp = mcp_opcode().map(Opcode::Mcp);
    let op_mcpi = mcpi_opcode().map(Opcode::Mcpi);
    let op_meq = meq_opcode().map(Opcode::Meq);
    let op_sb = sb_opcode().map(Opcode::Sb);
    let op_sw = sw_opcode().map(Opcode::Sw);
    let op_bal = bal_opcode().map(Opcode::Bal);
    let op_bhei = bhei_opcode().map(Opcode::Bhei);
    let op_bhsh = bhsh_opcode().map(Opcode::Bhsh);
    let op_burn = burn_opcode().map(Opcode::Burn);
    let op_call = call_opcode().map(Opcode::Call);
    let op_cb = cb_opcode().map(Opcode::Cb);
    let op_ccp = ccp_opcode().map(Opcode::Ccp);
    let op_croo = croo_opcode().map(Opcode::Croo);
    let op_csiz = csiz_opcode().map(Opcode::Csiz);
    let op_ldc = ldc_opcode().map(Opcode::Ldc);
    let op_log = log_opcode().map(Opcode::Log);
    let op_logd = logd_opcode().map(Opcode::Logd);
    let op_mint = mint_opcode().map(Opcode::Mint);
    let op_retd = retd_opcode().map(Opcode::Retd);
    let op_rvrt = rvrt_opcode().map(Opcode::Rvrt);
    let op_sldc = sldc_opcode().map(Opcode::Sldc);
    let op_srw = srw_opcode().map(Opcode::Srw);
    let op_srwq = srwq_opcode().map(Opcode::Srwq);
    let op_sww = sww_opcode().map(Opcode::Sww);
    let op_swwq = swwq_opcode().map(Opcode::Swwq);
    let op_tr = tr_opcode().map(Opcode::Tr);
    let op_tro = tro_opcode().map(Opcode::Tro);
    let op_ecr = ecr_opcode().map(Opcode::Ecr);
    let op_k256 = k256_opcode().map(Opcode::K256);
    let op_s256 = s256_opcode().map(Opcode::S256);
    let op_xil = xil_opcode().map(Opcode::Xil);
    let op_xis = xis_opcode().map(Opcode::Xis);
    let op_xol = xol_opcode().map(Opcode::Xol);
    let op_xos = xos_opcode().map(Opcode::Xos);
    let op_xwl = xwl_opcode().map(Opcode::Xwl);
    let op_xws = xws_opcode().map(Opcode::Xws);
    let op_flag = flag_opcode().map(Opcode::Flag);
    let op_gm = gm_opcode().map(Opcode::Gm);

    or! {
        op_addi,
        op_add,
        op_andi,
        op_and,
        op_divi,
        op_div,
        op_eq,
        op_expi,
        op_exp,
        op_gt,
        op_lt,
        op_mlog,
        op_modi,
        op_mod,
        op_move,
        op_mroo,
        op_muli,
        op_mul,
        op_noop,
        op_not,
        op_ori,
        op_or,
        op_slli,
        op_sll,
        op_srli,
        op_srl,
        op_subi,
        op_sub,
        op_xori,
        op_xor,
        op_cimv,
        op_ctmv,
        op_ji,
        op_jnei,
        op_aloc,
        op_cfei,
        op_cfsi,
        op_lb,
        op_lw,
        op_mcli,
        op_mcl,
        op_mcpi,
        op_mcp,
        op_meq,
        op_sb,
        op_bal,
        op_bhei,
        op_bhsh,
        op_burn,
        op_call,
        op_cb,
        op_ccp,
        op_croo,
        op_csiz,
        op_ldc,
        op_logd,
        op_log,
        op_mint,
        op_retd,
        op_ret,
        op_rvrt,
        op_sldc,
        op_srwq,
        op_srw,
        op_sww,
        op_swwq,
        op_sw,
        op_tro,
        op_tr,
        op_ecr,
        op_k256,
        op_s256,
        op_xil,
        op_xis,
        op_xol,
        op_xos,
        op_xwl,
        op_xws,
        op_flag,
        op_gm,
    }
    .try_map_with_span(|value_opt: Option<Opcode>, span| {
        value_opt.ok_or_else(|| ParseError::UnknownOpcode { span })
    })
}

pub fn asm_arg() -> impl Parser<Output = AsmArg> + Clone {
    let immediate = asm_immediate().map(AsmArg::Immediate);
    let register = ident().map(AsmArg::Register);

    immediate.or(register)
}

