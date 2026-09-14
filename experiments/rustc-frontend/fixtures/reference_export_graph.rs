fn main() {
    use std::io::BufRead;
    for line in std::io::stdin().lock().lines() {
        let input: i32 = line.unwrap().parse().unwrap();
        println!("{}", export_graph_model::score(input));
    }
}
