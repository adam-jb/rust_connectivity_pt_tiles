use typed_index_collections::TiVec;

use crate::read_file_funcs::{read_vec_as_array_multiplier, read_vec_as_array_usize};

use crate::structs::{
    Cost, DestinationReached, Multiplier, NodeID,Angle,
    Score, SecondsPastMidnight, SubpurposeScore, PURPOSES_COUNT, SUBPURPOSES_COUNT,
};

pub fn initialise_subpurpose_purpose_lookup() -> [usize; SUBPURPOSES_COUNT] {
    
    let subpurpose_purpose_lookup: [usize; SUBPURPOSES_COUNT] =
        read_vec_as_array_usize("subpurpose_to_purpose_integer");
    
    return subpurpose_purpose_lookup;
}


// update to accept mode name
pub fn initialise_score_multiplers(mode: &str) -> [Multiplier; SUBPURPOSES_COUNT] {
    
    let contents_filename = format!("score_multipliers_{}", mode);
        
    let multipliers_this_mode: [Multiplier; SUBPURPOSES_COUNT] = read_vec_as_array_multiplier(&contents_filename);
        //deserialize_bincoded_file(&contents_filename);
    
    return multipliers_this_mode;
}



pub fn get_time_of_day_index(trip_start_seconds: SecondsPastMidnight) -> usize {
    let mut time_of_day_ix = 0;
    if trip_start_seconds > SecondsPastMidnight(3600 * 10) {
        time_of_day_ix = 1;
    }
    if trip_start_seconds > SecondsPastMidnight(3600 * 16) {
        time_of_day_ix = 2;
    }
    if trip_start_seconds > SecondsPastMidnight(3600 * 19) {
        time_of_day_ix = 3;
    }
    time_of_day_ix as usize
}

pub fn add_to_subpurpose_scores_for_node_reached(subpurpose_scores: &mut [Score; SUBPURPOSES_COUNT],
                          node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,
                          subpurpose_purpose_lookup: &[usize; SUBPURPOSES_COUNT],
                          travel_time_relationships: &[Multiplier],
                          seconds_so_far: usize,
                          node_id: NodeID,
                          )  {
    for SubpurposeScore {
            subpurpose_ix,
            subpurpose_score,
    } in node_values_2d[node_id].iter()
    {
        let vec_start_pos_this_purpose = subpurpose_purpose_lookup[*subpurpose_ix] * 3601;
        let travel_time_multiplier = travel_time_relationships[vec_start_pos_this_purpose + seconds_so_far];
        let score_to_add = subpurpose_score.multiply(travel_time_multiplier);
        subpurpose_scores[*subpurpose_ix] += score_to_add;
    }
}


pub fn calculate_purpose_scores_from_subpurpose_scores(
        subpurpose_scores: &[Score; SUBPURPOSES_COUNT],
        subpurpose_purpose_lookup: &[usize; SUBPURPOSES_COUNT],
        score_multipler: &[Multiplier; SUBPURPOSES_COUNT],
    ) -> [Score; PURPOSES_COUNT] {

    let mut overall_purpose_scores: [Score; PURPOSES_COUNT] = [Score(0.0); PURPOSES_COUNT];
    for subpurpose_ix in 0..subpurpose_scores.len() {

        // Apply score_multipler and apply logarithm to get subpurpose level scores
        let mut subpurpose_score = subpurpose_scores[subpurpose_ix]
            .multiply(score_multipler[subpurpose_ix])
            .ln();

        // make negative values zero: this corrects for an effect of using log()
        if subpurpose_score < Score(0.0) {
            subpurpose_score = Score(0.0);
        }

        // add to purpose level scores
        let purpose_ix = subpurpose_purpose_lookup[subpurpose_ix];
        overall_purpose_scores[purpose_ix] += subpurpose_score;
    }
    overall_purpose_scores
}



pub fn get_cost_of_turn(
    angle_leaving_node_from: Angle,
    angle_arrived_from: Angle,
    time_costs_turn: &[Cost; 4], 
    ) -> Cost {

    let time_turn_previous_node: Cost;
    let angle_turn_previous_node: Angle;

    if angle_leaving_node_from < angle_arrived_from {
        angle_turn_previous_node = angle_leaving_node_from + Angle(360) - angle_arrived_from;
    } else {
        angle_turn_previous_node = angle_leaving_node_from -  angle_arrived_from;
    }

    // right turn
    if Angle(45) <= angle_turn_previous_node && angle_turn_previous_node < Angle(135) {
        time_turn_previous_node = time_costs_turn[1];
    // u turn
    } else if Angle(135) <= angle_turn_previous_node && angle_turn_previous_node < Angle(225) {
        time_turn_previous_node = time_costs_turn[2];
    // left turn
    } else if Angle(225) <= angle_turn_previous_node && angle_turn_previous_node < Angle(315) {
       time_turn_previous_node = time_costs_turn[3];
    // no turn
    } else {
        time_turn_previous_node = time_costs_turn[0];
    }

    time_turn_previous_node
}



pub fn extract_od_pairs(
    destinations_reached: Vec<DestinationReached>,
) -> Vec<[usize; 2]> {

    let mut od_pairs: Vec<[usize; 2]> = Vec::new();
    for destination_reached in destinations_reached {

        od_pairs.push([destination_reached.node.0, destination_reached.cost.0]);

    }
    od_pairs
}

