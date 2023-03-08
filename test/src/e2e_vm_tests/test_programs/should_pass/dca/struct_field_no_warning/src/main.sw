script;

struct CallData {
  id: u64,
}

pub struct Proposal {
  call_data: CallData,
}

fn main(p: Proposal) -> u64 {
  p.call_data.id
}
