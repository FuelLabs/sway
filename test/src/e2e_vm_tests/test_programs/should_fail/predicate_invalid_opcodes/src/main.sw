predicate;

fn main() -> bool {
  asm(r1, r2, r3) {
    bal r1 r2 r3;
  };

  asm(r1) {
    bhei r1;
  };

  asm(r1, r2) {
    bhsh r1 r2;
  };

  asm(r1) {
    burn r1;
  };

  asm(r1, r2, r3, r4) {
    call r1 r2 r3 r4;
  };

  asm(r1) {
    cb r1;
  };

  asm(r1, r2, r3, r4) {
    ccp r1 r2 r3 r4;
  };

  asm(r1, r2) {
    croo r1 r2;
  };

  asm(r1, r2) {
    csiz r1 r2;
  };

  asm(r1) {
    gm r1 i1;
  }

  asm(r1) {
    gm r1 i2;
  }

  // should not throw an error.
  asm(r1) {
    gm r1 i3;
  };

  asm(r1, r2, r3) {
    ldc r1 r2 r3;
  }

  asm(r1, r2, r3, r4) {
    log r1 r2 r3 r4;
  }

  asm(r1, r2, r3, r4) {
    logd r1 r2 r3 r4;
  }

  asm(r1) {
    mint r1;
  }

  // retd: There is no way of testing
  // rvrt: It is allowed and used to abort predicates.

  asm(r1, r2, r3, r4) {
    smo r1 r2 r3 r4;
  }

  asm(r1, r2, r3) {
    srw r1 r2 r3;
  }

  asm(r1, r2, r3, r4) {
    srwq r1 r2 r3 r4;
  }

  asm(r1, r2, r3) {
    sww r1 r2 r3;
  }

  asm(r1, r2, r3, r4) {
    swwq r1 r2 r3 r4;
  }

  asm(r1, r2) {
    time r1 r2;
  }

  asm(r1, r2, r3) {
    tr r1 r2 r3;
  }

  asm(r1, r2, r3, r4) {
    tro r1 r2 r3 r4;
  }

  // While loop compiles to JI with backward offset
  let mut i = 0;
  while i < 30 {
    i += 1;
  }

  true
}