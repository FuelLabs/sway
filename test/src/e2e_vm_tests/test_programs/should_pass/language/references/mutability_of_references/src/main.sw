script;

fn main() -> u64 {
    let r = &1u8;
    let mut m_r = &1u8;

    let r_ptr = asm(r: r) { r: raw_ptr };
    let m_r_ptr_01 = asm(r: m_r) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);

    m_r = &(1u8 + 1);

    let m_r_ptr_02 = asm(r: m_r) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);
    assert(r_ptr != m_r_ptr_02);
    assert(m_r_ptr_01 != m_r_ptr_02);

    m_r = r;

    let m_r_ptr_03 = asm(r: m_r) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);
    assert(r_ptr != m_r_ptr_02);
    assert(r_ptr == m_r_ptr_03);

    42
}
