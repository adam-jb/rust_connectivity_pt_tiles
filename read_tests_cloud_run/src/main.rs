use fs_err::File;
use std::time::Instant;
use std::io::BufWriter;
use actix_web::{get, web, App, HttpServer};

use common::structs::{NodeRoute, NodeWalk, SubpurposeScore};
use common::read_file_funcs::{deserialize_bincoded_file, read_files_parallel_inc_node_values};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    #[allow(deprecated)]
    HttpServer::new(move || {
        App::new()
            // TODO Fix before deploying for real!
            .wrap(actix_cors::Cors::permissive())
            .data(web::JsonConfig::default().limit(1024 * 1024 * 500)) // allow POST'd JSON payloads up to 500mb
            .service(run_tests)
    })
    .bind(("0.0.0.0", 7328))?
    .run()
    .await
}    

#[get("/")]
async fn index() -> String {
    format!("App is listening")
}

#[get("/run_tests/")]
async fn run_tests() -> String {
    
    let year = 2022;
    
    let now = Instant::now();
    let (_node_values_2d, graph_walk, graph_routes) = read_files_parallel_inc_node_values(year);
    println!(
        "Standard loading took {:?}",
        now.elapsed()
    );
    
    // **** Creating for 3 chunks of each file
    let chunk_count = 3;
    let graph_walk_chunk_size = 1 + graph_walk.len() / chunk_count; 
    let graph_walk_in_chunks: Vec<_> = graph_walk.chunks(graph_walk_chunk_size)
                                       .map(|chunk| chunk.to_vec())
                                       .collect();

    for (i, chunk) in graph_walk_in_chunks.iter().enumerate() {
        println!("Chunk {} len: {}", i+1, chunk.len());
        let filename = format!("serialised_data/graph_pt_walk_chunk_{}.bin", i+1);
        let file = BufWriter::new(File::create(filename).unwrap());
        bincode::serialize_into(file, &chunk).unwrap();
        println!("Walk Chunk {} serialised", i + 1);
    }
    
    let graph_route_chunk_size = graph_routes.len() / chunk_count; 
    let graph_route_in_chunks: Vec<_> = graph_routes.chunks(graph_route_chunk_size)
                                       .map(|chunk| chunk.to_vec())
                                       .collect();

    for (i, chunk) in graph_route_in_chunks.iter().enumerate() {
        println!("Chunk {} len: {}", i+1, chunk.len());
        let filename = format!("serialised_data/graph_pt_routes_chunk_{}.bin", i+1);
        let file = BufWriter::new(File::create(filename).unwrap());
        bincode::serialize_into(file, &chunk).unwrap();
        println!("Routes Chunk {} serialised", i + 1);
    }
    
    // Func to read starts here
    let now = Instant::now();
    
    // if editing: make sure you get the types being deserialised into right: the compiler may panic without telling you why if you do
    let (((mut graph_walk1, mut graph_walk2), mut graph_walk3), ((mut graph_routes1, mut graph_routes2), (mut graph_routes3, _node_values_2d))) =
    rayon::join(
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_2")),
                    )
                },
                || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_3")),
            )
        },
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_2")),
                    )
                },
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_3")),
                        || deserialize_bincoded_file::<Vec<Vec<SubpurposeScore>>>(&format!("sparse_node_values_6am_{year}_2d")),
                    )
                },
            )
        }
    );
    
    println!(
        "Parallel loading for files without extend took {:?}",
        now.elapsed()
    );
    
    graph_walk1.reserve(graph_walk.len());
    graph_walk1.append(&mut graph_walk2);
    graph_walk1.append(&mut graph_walk3);
    
    graph_routes1.reserve(graph_routes.len() + 10);
    graph_routes1.append(&mut graph_routes2);
    graph_routes1.append(&mut graph_routes3);
    
    println!(
        "Parallel loading for files with {} chunks and extend took {:?}",
        3, 
        now.elapsed()
    );
    
    // return (graph_walk1, graph_routes1, _node_values_2d)
    
    
    
    // **** Creating for 4 chunks per file, and reading node values in serial
    let chunk_count = 4;
    let graph_walk_chunk_size = 1 + graph_walk.len() / chunk_count; 
    let graph_walk_in_chunks: Vec<_> = graph_walk.chunks(graph_walk_chunk_size)
                                       .map(|chunk| chunk.to_vec())
                                       .collect();

    for (i, chunk) in graph_walk_in_chunks.iter().enumerate() {
        println!("Chunk {} len: {}", i, chunk.len());
        let filename = format!("serialised_data/graph_pt_walk_chunk_{}.bin", i+1);
        let file = BufWriter::new(File::create(filename).unwrap());
        bincode::serialize_into(file, &chunk).unwrap();
        println!("Chunk {} serialised", i + 1);
    }
    
    let graph_route_chunk_size = graph_routes.len() / chunk_count; 
    let graph_route_in_chunks: Vec<_> = graph_routes.chunks(graph_route_chunk_size)
                                       .map(|chunk| chunk.to_vec())
                                       .collect();

    for (i, chunk) in graph_route_in_chunks.iter().enumerate() {
        println!("Chunk {} len: {}", i, chunk.len());
        let filename = format!("serialised_data/graph_pt_routes_chunk_{}.bin", i+1);
        let file = BufWriter::new(File::create(filename).unwrap());
        bincode::serialize_into(file, &chunk).unwrap();
        println!("Chunk {} serialised", i + 1);
    }
    
    let (((mut graph_walk1, mut graph_walk2), (mut graph_walk3, mut graph_walk4)), ((mut graph_routes1, mut graph_routes2), (mut graph_routes3, mut graph_routes4))) =
    rayon::join(
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_2")),
                    )
                },
                || {rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_3")),
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_4")),
                    )
                },
            )
        },
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_2")),
                    )
                },
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_3")),
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_4")),
                    )
                },
            )
        }
    );
    
    println!(
        "Parallel loading for files without extend took {:?}",
        now.elapsed()
    );
    
    graph_walk1.reserve(graph_walk.len());
    graph_walk1.append(&mut graph_walk2);
    graph_walk1.append(&mut graph_walk3);
    graph_walk1.append(&mut graph_walk4);
    
    graph_routes1.reserve(graph_routes.len());
    graph_routes1.append(&mut graph_routes2);
    graph_routes1.append(&mut graph_routes3);
    graph_routes1.append(&mut graph_routes4);
    
    println!(
        "Parallel loading for files with extend ignoring node_values_2d took {:?}",
        now.elapsed()
    );
    
    let _node_values_2d: Vec<Vec<SubpurposeScore>> = deserialize_bincoded_file(&format!("sparse_node_values_6am_{year}_2d"));
    
    println!(
        "Parallel loading for files and extend with {} chunks took {:?}",
        4,
        now.elapsed()
    );
        
    
    // Checking for one chunk
    let now = Instant::now();
    let _graph_walk1: Vec<NodeWalk> = deserialize_bincoded_file(&format!("graph_pt_walk_chunk_1"));
    println!(
        "Serial loading for one chunk out of 4 took {:?}",
        now.elapsed()
    );
    
    let results = vec![1,2,3];
    serde_json::to_string(&results).unwrap()
    
}

