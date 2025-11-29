use clap::Parser;

#[derive(Parser)]
#[command(version)]
#[command(about = "Calculates all possible full methods given a number of bells")]
#[command(long_about = None)]
struct Args {
  number_of_bells: usize,
}

fn main() {
  let args = Args::parse();

  let graph = rusty_bells::PermutationGraph::new(args.number_of_bells);

  let method_iterator = graph.get_valid_full_methods();

  for (index, method) in method_iterator.enumerate() {
    rusty_bells::print_method(index, &method);
  }
}
