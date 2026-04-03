predicate;

fn main() -> bool {
  let bal = asm(r1, r2: 0, r3: 0) {
    bal r1 r2 r3;
    r1: u64
  };

  let bhei = asm(r1) {
    bhei r1;
    r1: u64
  };

  asm(r1: 0, r2: 0) {
    bhsh r1 r2;
  };

  asm(r1: 0, r2: 0) {
    burn r1 r2;
  };

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    call r1 r2 r3 r4;
  };

  asm(r1: 0) {
    cb r1;
  };

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    ccp r1 r2 r3 r4;
  };

  asm(r1: 0, r2: 0) {
    croo r1 r2;
  };

  let csiz = asm(r1, r2: 0) {
    csiz r1 r2;
    r1: u64
  };

  let is_ext_caller = asm(r1) {
    gm r1 i1;
    r1: bool
  };

  let caller_id = asm(r1) {
    gm r1 i2;
    r1: u64
  };

  // should not throw an error.
  let pred_index = asm(r1) {
    gm r1 i3;
    r1: u64
  };

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    log r1 r2 r3 r4;
  }

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    logd r1 r2 r3 r4;
  }

  asm(r1: 0, r2: 0) {
    mint r1 r2;
  }

  // retd: There is no way of testing
  // rvrt: It is allowed and used to abort predicates.

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    smo r1 r2 r3 r4;
  }

  // cannot test storage opcodes due to needing to annotate main
  // with #[storage(read, write)] which is not allowed for predicates
  /*
  asm(r1: 0, r2: 0, r3) {
    srw r1 r2 r3 i0;
  }

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    srwq r1 r2 r3 r4;
  }

  asm(r1: 0, r2: 0, r3) {
    sww r1 r2 r3;
  }

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    swwq r1 r2 r3 r4;
  }
  */

  let time = asm(r1, r2: 0) {
    time r1 r2;
    r1: u64
  };

  asm(r1: 0, r2: 0, r3: 0) {
    tr r1 r2 r3;
  }

  asm(r1: 0, r2: 0, r3: 0, r4: 0) {
    tro r1 r2 r3 r4;
  }

  bal == 0 && bhei == 0 && csiz == 0 && is_ext_caller && pred_index == 0 && caller_id != 0 && time != 0
}