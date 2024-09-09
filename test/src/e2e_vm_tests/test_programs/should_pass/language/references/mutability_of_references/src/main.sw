script;

fn main() -> u64 {
    let mut r = &1u8;
    let mut m_r = &1u8;
    let mut m_r_m = &mut 1u8;

    let r_ptr = asm(r: r) { r: raw_ptr };
    let m_r_ptr_01 = asm(r: m_r) { r: raw_ptr };
    let m_r_m_ptr_01 = asm(r: m_r_m) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);
    assert(m_r_ptr_01 != m_r_m_ptr_01);
    assert(m_r_m_ptr_01 != r_ptr);

    assert(*r == *m_r);
    assert(*r == *m_r_m);

    m_r = &(1u8 + 1);
    m_r_m = &mut (1u8 + 1);

    let m_r_ptr_02 = asm(r: m_r) { r: raw_ptr };
    let m_r_m_ptr_02 = asm(r: m_r_m) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);
    assert(r_ptr != m_r_ptr_02);
    assert(r_ptr != m_r_m_ptr_01);
    assert(r_ptr != m_r_m_ptr_02);
    assert(m_r_ptr_01 != m_r_ptr_02);
    assert(m_r_m_ptr_01 != m_r_m_ptr_02);

    assert(*r != *m_r);
    assert(*r != *m_r_m);

    m_r = r;

    let m_r_ptr_03 = asm(r: m_r) { r: raw_ptr };

    assert(r_ptr != m_r_ptr_01);
    assert(r_ptr != m_r_ptr_02);
    assert(r_ptr == m_r_ptr_03);

    assert(*r == *m_r);

    42
}
