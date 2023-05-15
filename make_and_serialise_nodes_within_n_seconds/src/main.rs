fn main() {
    println!("Hello, world!");
}
/*
let seconds_travel_for_destination_clustering = 120;

    // comment this out to not make the lookup of nodes which are near other nodes
    // this is big preprocessing stage (~90mins with 8cores for 120 seconds)
    if false {
        for time_seconds in [120, 180, 240, 300] {
            let now = Instant::now();
            let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(year);
            make_and_serialise_nodes_within_n_seconds::make_and_serialise_nodes_within_n_seconds(
                Cost(time_seconds),
                graph_walk,
                graph_pt,
            );
            println!(
                "All nearby nodes for {} seconds took {:?} seconds",
                time_seconds,
                now.elapsed()
            );
        }
    }
    */