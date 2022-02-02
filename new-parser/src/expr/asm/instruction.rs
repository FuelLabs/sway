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


    op_addi
    .or(op_add)
    .or(op_andi)
    .or(op_and)
    .or(op_divi)
    .or(op_div)
    .or(op_eq)
    .or(op_expi)
    .or(op_exp)
    .or(op_gt)
    .or(op_lt)
    .or(op_mlog)
    .or(op_modi)
    .or(op_mod)
    .or(op_move)
    .or(op_mroo)
    .or(op_muli)
    .or(op_mul)
    .or(op_noop)
    .or(op_not)
    .or(op_ori)
    .or(op_or)
    .or(op_slli)
    .or(op_sll)
    .or(op_srli)
    .or(op_srl)
    .or(op_subi)
    .or(op_sub)
    .or(op_xori)
    .or(op_xor)
    .or(op_cimv)
    .or(op_ctmv)
    .or(op_ji)
    .or(op_jnei)
    .or(op_aloc)
    .or(op_cfei)
    .or(op_cfsi)
    .or(op_lb)
    .or(op_lw)
    .or(op_mcli)
    .or(op_mcl)
    .or(op_mcpi)
    .or(op_mcp)
    .or(op_meq)
    .or(op_sb)
    .or(op_bal)
    .or(op_bhei)
    .or(op_bhsh)
    .or(op_burn)
    .or(op_call)
    .or(op_cb)
    .or(op_ccp)
    .or(op_croo)
    .or(op_csiz)
    .or(op_ldc)
    .or(op_logd)
    .or(op_log)
    .or(op_mint)
    .or(op_retd)
    .or(op_ret)
    .or(op_rvrt)
    .or(op_sldc)
    .or(op_srwq)
    .or(op_srw)
    .or(op_sww)
    .or(op_swwq)
    .or(op_sw)
    .or(op_tro)
    .or(op_tr)
    .or(op_ecr)
    .or(op_k256)
    .or(op_s256)
    .or(op_xil)
    .or(op_xis)
    .or(op_xol)
    .or(op_xos)
    .or(op_xwl)
    .or(op_xws)
    .or(op_flag)
    .or(op_gm)
}

pub fn asm_arg() -> impl Parser<Output = AsmArg> + Clone {
    let immediate = asm_immediate().map(AsmArg::Immediate);
    let register = ident().map(AsmArg::Register);

    immediate.or(register)
}

