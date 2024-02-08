// target-fuelvm
contract;

struct S {
    i: u64,
    j: u64,
}


abi TestContract {
    fn foo(a: [u64; 10]) -> u64;
    fn bar(s: S) -> u64;
    fn boo(s: [S; 4]) -> u64;
}

impl TestContract for Contract {
     fn foo(a: [u64; 10]) -> u64 {
        a[9]
     }
     fn bar(s: S) -> u64 {
        s.j
     }
     fn boo(sa: [S; 4]) -> u64 {
        sa[2].j
     }
}

// ::check-ir::
// check: entry fn bar<28d8fbc1>
// check: entry fn boo<4938f594>
// check: entry fn foo<ab6ec620>

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+
// regex: DATA=data_\d+

// not: addi $REG $REG $IMM
// check: lw   $REG $REG i1 

// not: movi $REG $IMM
// not: load $REG $DATA
// not: mul  $REG $REG $REG
// not: add $REG $REG $REG
// not: addi $REG $REG $IMM
// check: lw   $REG $REG i5

// not: load $REG $DATA
// not: mul  $REG $REG $REG
// not: add  $REG $REG $REG
// check: lw   $REG $REG i9 

