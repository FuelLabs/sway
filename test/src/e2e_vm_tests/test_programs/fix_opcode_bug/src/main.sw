script;

// This file tests different kinds of ASM generation and parsing.

fn abi_x() {}
fn enum_x() {}
fn fn_x() {}
fn let_x() {}
fn match_x() {}
fn return_x() {}
fn struct_x() {}
fn trait_x() {}
fn use_x() {}
fn while_x() {}

fn asm_x() {}
fn as_x() {}
fn contract_x() {}
fn dep_x() {}
fn deref_x() {}
fn false_x() {}
fn for_x() {}
fn i64_x() {}
fn impl_x() {}
fn library_x() {}
fn mut_x() {}
fn predicate_x() {}
fn pub_x() {}
fn ref_x() {}
fn script_x() {}
fn self_x() {}
fn str_x() {}
fn true_x() {}
fn u64_x() {}
fn where_x() {}

fn blockheight() -> u64 {
	asm(r1) {
		bhei r1;
		r1: u64
	}
}

struct GasCounts {
	global_gas: u64,
	context_gas: u64
}

fn get_gas() -> GasCounts {
  GasCounts {
		global_gas: asm() {
			ggas
		},
		context_gas: asm() {
			cgas
		}
	}
}

fn main() -> u32 {
	let block_height = blockheight();
	let remaining_gas = get_gas();
  return 6u32;
}
