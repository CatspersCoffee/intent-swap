library;

use std::{
    bytes::Bytes,
    math::*,
    option::Option,
    string::String,
};
use std::*;
use std::bytes_conversions::{b256::*, u64::*};
use std::primitive_conversions::{u16::*, u32::*, u64::*};

use helpers::numeric_utils::*;



/// A basic struct to store information for either an
/// transaction input or output.
pub struct InpOut {
    pub assetid: b256,
    pub amount: Option<u64>,
    pub amount32: Option<b256>,
    pub uxtoid: Option<b256>,
    pub owner: Option<Address>,
}

impl InpOut {
    pub fn new(
        assetid: b256,
        amountu64: Option<u64>,
        amountbytes32: Option<b256>,
        uxtoid: Option<b256>,
        owner: Option<Address>,
    ) -> InpOut {
        InpOut {
            assetid: assetid,
            amount: amountu64,
            amount32: amountbytes32,
            uxtoid: uxtoid,
            owner: owner,
        }
    }
}



/// Reorders transaction inputs to match the sequence of expected UTXOs.
/// Any inputs with UTXOs not in the expected sequence are appended to the end.
/// Expected zero UTXOs are skipped in the matching process.
///
/// # Arguments
/// * `tx_inputs` - Mutable reference to vector of transaction inputs to reorder
/// * `expected_utxos` - Reference to array of expected UTXO order
pub fn reorder_inputs(
    ref mut tx_inputs: Vec<InpOut>,
    expected_utxos: [b256; 5],
) {
    let mut reordered: Vec<InpOut> = Vec::new();
    let mut remaining: Vec<InpOut> = Vec::new();

    // For each expected UTXO that is not zero
    let mut i = 0;
    while i < 5 {
        let expected_utxo = expected_utxos[i];

        // Skip if the expected UTXO is zero
        if expected_utxo != b256::zero() {
            let mut j = 0;
            let mut found = false;

            while j < tx_inputs.len() {
                if let Some(utxo_id) = tx_inputs.get(j).unwrap().uxtoid {
                    if utxo_id == expected_utxo {
                        // Found matching input, add to reordered list
                        reordered.push(tx_inputs.remove(j));
                        found = true;
                        break;
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }

    // Move all remaining inputs to the remaining vector
    while !tx_inputs.is_empty() {
        remaining.push(tx_inputs.remove(0));
    }

    // First push all reordered items back
    while !reordered.is_empty() {
        tx_inputs.push(reordered.remove(0));
    }

    // Then push all remaining items
    while !remaining.is_empty() {
        tx_inputs.push(remaining.remove(0));
    }
}

/// Represents the result of comparing aggregated assets with Expected Input and Output assets.
///
/// This enum encapsulates the outcome of the asset comparison process,
/// providing either detailed information about successful matches or
/// an error code indicating why the comparison failed.
///
/// # Variants
///
/// ## Success(([b256; 3], u64, b256, u64, bool))
///
/// Indicates a successful comparison. The tuple contains:
/// 1. b256: A b256 value representing the matched input asset.
/// 2. u64: The total count of matched input assets, including duplicates.
/// 3. b256: The matched output asset. b256::zero() if no output asset was matched.
/// 4. u64: The count of matched output assets (0 or 1).
/// 5. bool: Indicates whether all matched input assets are of the same type.
///    - true: All matched input assets have the same asset ID.
///    - false: Matched input assets have different asset IDs.
///
/// ## Fail(u64)
///
/// Indicates that the comparison failed. The u64 value is an error code:
/// - 1: An asset in the aggregated assets was not found in the expected inputs or output.
/// - 2: An asset specified in the expected inputs was not found in the aggregated assets.
/// - 3: The output asset specified in the expected_asset_out was not found in the aggregated assets.
///
/// # Usage
///
/// This enum is typically used as the return type for functions that compare
/// sets of assets, such as `compare_assets_simplified`. It allows for rich
/// error handling and detailed reporting of successful comparisons.
///
/// # Example
///
/// ```
/// match compare_assets_simplified(agg_assets, expected_assets_in, expected_asset_out) {
///     CompareResult::Success((matches, count, out_match, out_count, all_same)) => {
///         // Handle successful comparison
///     },
///     CompareResult::Fail(error_code) => {
///         // Handle failure based on error code
///     }
/// }
/// ```
pub enum CompareResult {
    Success: ((b256, u64, b256, u64, bool)),
    Fail: (u64),
}


/// Compares aggregated assets with expected assets and returns the result.
///
/// This function checks if the assets specified in the expected arrays (both inputs and output)
/// are present in the aggregated assets. It ignores any aggregated assets that are not
/// specified in the expected arrays.
///
/// # Arguments
///
/// * `agg_assets` - An array of 3 b256 values representing the aggregated assets.
/// * `expected_assets_in` - An array of 5 b256 values representing the expected input assets.
/// * `expected_asset_out` - A b256 value representing the expected output asset.
///
/// # Returns
///
/// A `CompareResult` enum with two variants:
///
/// ## Success((b256, u64, b256, u64, bool))
///
/// Indicates a successful comparison. The tuple contains:
/// 1. b256: The first matched input asset from expected_assets_in found in agg_assets.
/// 2. u64: The total count of matched input assets, including duplicates.
/// 3. b256: The matched output asset. b256::zero() if no output asset was matched.
/// 4. u64: The count of matched output assets (0 or 1).
/// 5. bool: Indicates whether all matched input assets are of the same type.
///    - true: All matched input assets have the same asset ID.
///    - false: Matched input assets have different asset IDs.
///
/// ## Fail(u64)
///
/// Indicates that the comparison failed. The u64 value is an error code:
/// - 1: This error code is not used in the current implementation.
/// - 2: An asset specified in the expected inputs was not found in the aggregated assets.
/// - 3: The asset specified in the expected output was not found in the aggregated assets.
///
/// # Behavior
///
/// - The function only considers assets from agg_assets that are present in expected_assets_in or match expected_asset_out.
/// - Assets in agg_assets that are not in expected_assets_in or don't match expected_asset_out are ignored.
/// - If all specified expected assets (inputs and output) are found in agg_assets, it returns a Success result.
/// - If any specified expected asset is not found in agg_assets, it returns a Fail result with the appropriate error code.
///
/// # Example
///
/// ```sway
/// let agg_assets = [asset1, asset2, asset3];
/// let expected_assets_in = [asset1, asset1, asset1, b256::zero(), b256::zero()];
/// let expected_asset_out = asset2;
///
/// match compare_assets_simplified(agg_assets, expected_assets_in, expected_asset_out) {
///     CompareResult::Success((in_match, in_count, out_match, out_count, all_same_type)) => {
///         // Handle successful comparison
///     },
///     CompareResult::Fail(error_code) => {
///         // Handle failure based on error code
///     }
/// }
/// ```
pub fn compare_assets_simplified(
    agg_assets: [b256; 3],
    expected_assets_in: [b256; 5],
    expected_asset_out: b256,
) -> CompareResult {
    let mut matched_input = b256::zero();
    let mut input_count = 0;
    let mut output_match = b256::zero();
    let mut output_count = 0;
    let mut all_same_type = true;

    // Check aggregated assets
    let mut i = 0;
    while i < 3 {
        let asset = agg_assets[i];
        if asset != b256::zero() {
            let found_in_input = contains_val_5(expected_assets_in, asset);
            let found_in_output = asset == expected_asset_out;

            if found_in_input {
                if matched_input == b256::zero() {
                    matched_input = asset;
                } else if matched_input != asset {
                    all_same_type = false;
                }

                // Count occurrences in expected_assets_in
                let mut j = 0;
                while j < 5 {
                    if expected_assets_in[j] == asset {
                        input_count += 1;
                    }
                    j += 1;
                }
            } else if found_in_output {
                output_match = asset;
                output_count = 1;
            }
            // If not found in input or output, we simply ignore it
        }
        i += 1;
    }

    // Check if all non-zero expected input assets are in agg_assets
    let mut j = 0;
    while j < 5 {
        let asset = expected_assets_in[j];
        if asset != b256::zero() {
            let mut k = 0;
            let mut found = false;
            while k < 3 {
                if agg_assets[k] == asset {
                    found = true;
                    break;
                }
                k += 1;
            }
            if !found {
                return CompareResult::Fail(2); // Asset in expected inputs not found in agg_assets
            }
        }
        j += 1;
    }

    // Check if expected output asset is in agg_assets
    if expected_asset_out != b256::zero() {
        let mut found = false;
        let mut k = 0;
        while k < 3 {
            if agg_assets[k] == expected_asset_out {
                found = true;
                break;
            }
            k += 1;
        }
        if !found {
            return CompareResult::Fail(3); // Output asset not found in agg_assets
        }
    }

    CompareResult::Success((matched_input, input_count, output_match, output_count, all_same_type))
}


pub enum AggregateResult {
    Success: (([b256; 3], [b256; 3])),
    Fail: (u64),
}

/// Aggregates assets and their amounts from a vector of InpOut structures.
///
/// # Arguments
///
/// * `tx_inputs`: A vector of InpOut structures containing asset IDs and amounts.
///
/// # Returns
///
/// * `AggregateResult`: Either Success with two arrays of 3 b256 elements each
///   (unique assets and their aggregated amounts), or Fail with an error code.
pub fn aggregate_assets(tx_inputs: Vec<InpOut>) -> AggregateResult {
    let mut unique_assets = [b256::zero(); 3];
    let mut aggregated_amountsu64 = [0u64; 3];
    let mut unique_count = 0;

    let mut i = 0;
    while i < tx_inputs.len() {
        let input = tx_inputs.get(i).unwrap();
        let asset = input.assetid;
        let amount = match input.amount {
            Some(val) => val,
            None => match input.amount32 {
                Some(val32) => b256_to_u64(val32),
                None => return AggregateResult::Fail(6666u64), // No valid amount provided
            },
        };

        let mut found = false;
        let mut j = 0;
        while j < unique_count {
            if unique_assets[j] == asset {
                if check_u64_addition_overflow(aggregated_amountsu64[j], amount) {
                    aggregated_amountsu64[j] += amount;
                    found = true;
                    break;
                } else {
                    return AggregateResult::Fail(3333u64); // Overflow error
                }
            }
            j += 1;
        }
        if !found {
            if unique_count < 3 {
                unique_assets[unique_count] = asset;
                aggregated_amountsu64[unique_count] = amount;
                unique_count += 1;
            } else {
                return AggregateResult::Fail(5555u64); // Too many unique assets
            }
        }
        i += 1;
    }

    let mut aggregated_amounts = [b256::zero(); 3];
    let mut k = 0;
    while k < 3 {
        aggregated_amounts[k] = to_b256((0, 0, 0, aggregated_amountsu64[k]));
        k += 1;
    }

    AggregateResult::Success((unique_assets, aggregated_amounts))
}


/// Finds the owner of the input asset specified by the expected asset.
///
/// # Arguments
///
/// * `tx_inputs`: Vector of transaction inputs
/// * `expected_asset_in`: The asset ID to look for in the inputs
///
/// # Returns
///
/// * `Option<Address>`: Returns Some(Address) if the asset is found and has a owner,
///                     None if the asset isn't found or has no owner
pub fn find_input_assets_owner(
    tx_inputs: Vec<InpOut>,
    expected_asset_in: b256,
) -> Option<Address> {
    // Skip if expected asset is zero
    if expected_asset_in == b256::zero() {
        return None;
    }
    // Look for the asset in tx_inputs
    let mut i = 0;
    while i < tx_inputs.len() {
        if tx_inputs.get(i).unwrap().assetid == expected_asset_in {
            return tx_inputs.get(i).unwrap().owner;
        }
        i += 1;
    }

    None
}


/// Compares aggregated input assets and amounts with expected input assets and amounts.
///
/// This function verifies that the aggregated amounts for each asset are sufficient
/// to cover the intended amounts specified in the expected inputs.
///
/// # Arguments
///
/// * `agg_assets`: An array of 3 b256 values representing the aggregated input asset IDs.
/// * `agg_amounts`: An array of 3 b256 values representing the aggregated input amounts.
/// * `expected_assets`: An array of 5 b256 values representing the expected input asset IDs.
/// * `expected_amounts`: An array of 5 b256 values representing the expected input amounts.
///
/// # Returns
///
/// * `bool`: Returns `true` if all aggregated amounts are greater than or equal to
///           the sum of corresponding expected amounts for each asset. Otherwise, returns `false`.
///
/// # Behavior
///
/// 1. Iterates through each non-zero asset in `agg_assets`.
/// 2. For each asset, sums up all corresponding amounts in `expected_amounts`.
/// 3. Compares the aggregated amount with the sum of intenexpected amounts.
/// 4. If any aggregated amount is less than the corresponding sum of expected amounts, returns `false`.
/// 5. If all checks pass, returns `true`.
///
/// # Note
///
/// - Assets present in `agg_assets` but not in `expected_assets` are ignored.
/// - The function assumes that `agg_assets` and `agg_amounts` have a length of 3,
///   while `expected_assets` and `expected_amounts` have a length of 5.
/// - Zero values in `agg_assets` are skipped.
/// - The function ensures that all non-zero assets in `expected_assets` are accounted for in `agg_assets`.
///
/// # Example
///
/// ```
/// let agg_assets = [asset1, asset2, b256::zero()];
/// let agg_amounts = [1520000000, 2000000000, 0];
/// let expected_assets = [asset1, asset1, b256::zero(), b256::zero(), b256::zero()];
/// let expected_amounts = [520000000, 1000000000, 0, 0, 0];
///
/// let result = compare_input_amounts(agg_assets, agg_amounts, expected_assets, expected_amounts);
/// assert_eq!(result, true);
/// ```
pub fn compare_input_amounts(
    agg_assets: [b256; 3],
    agg_amounts: [b256; 3],
    expected_assets: [b256; 5],
    expected_amounts: [b256; 5]
) -> bool {
    let mut i = 0;
    while i < 3 {
        if agg_assets[i] != b256::zero() {
            let agg_amount = agg_amounts[i];
            let mut expected_total = b256::zero();

            let mut j = 0;
            while j < 5 {
                if expected_assets[j] == agg_assets[i] {
                    let expected_total_res = add_b256(expected_total, expected_amounts[j]);
                    expected_total = expected_total_res.unwrap_or(b256::zero());
                }
                j += 1;
            }

            if agg_amount < expected_total {
                return false;
            }
        }
        i += 1;
    }

    // Check if all non-zero expected assets are accounted for in agg_assets
    let mut k = 0;
    while k < 5 {
        if expected_assets[k] != b256::zero() {
            let mut found = false;
            let mut l = 0;
            while l < 3 {
                if expected_assets[k] == agg_assets[l] {
                    found = true;
                    break;
                }
                l += 1;
            }
            if !found {
                return false;
            }
        }
        k += 1;
    }

    true
}


pub fn compare_output_amounts(
    agg_output_assets: [b256; 3],
    agg_output_amounts: [b256; 3],
    expected_output_asset: b256,
    expected_output_amount: b256,
    expected_input_assets: [b256; 5],
    tolerance_bps: u64  // Tolerance in basis points (1 bps = 0.01%)
) -> bool {
    let expected_amount = b256_to_u64(expected_output_amount);
    let mut found_expected_output = false;

    let mut i = 0;
    while i < 3 {
        if agg_output_assets[i] == expected_output_asset {
            found_expected_output = true;
            let agg_amount = b256_to_u64(agg_output_amounts[i]);

            // Check if this asset is NOT in the expected input assets (i.e., it's an output asset)
            let mut is_output_asset = true;
            let mut k = 0;
            while k < 5 {
                if expected_input_assets[k] == expected_output_asset {
                    is_output_asset = false;
                    break;
                }
                k += 1;
            }

            // log((i, is_output_asset, agg_output_assets[i]));

            if is_output_asset {
                // Calculate tolerance amount
                let tolerance_amount = (expected_amount * tolerance_bps) / 10000; // 10000 bps = 100%

                // Apply tolerance check for output assets
                let lower_bound = if expected_amount > tolerance_amount { expected_amount - tolerance_amount } else { 0 };
                let upper_bound = expected_amount + tolerance_amount;

                // log("Output asset check (agg_amount/upper_bound/lower_bound):");
                // log(agg_amount);
                // log(lower_bound);
                // log(upper_bound);

                if agg_amount < lower_bound || agg_amount > upper_bound {
                    return false;
                }
            } else {
                // For input assets, we don't need to check anything in the output
                // log("Input asset, skipping check");
            }
            break;
        }
        i += 1;
    }

    // Check if the expected output asset was found in agg_output_assets
    if !found_expected_output {
        return false; // Expected output asset not found in agg_output_assets
    }

    true
}


/// Checks if all UTXOs in expected_utxos are present in tx_inputs and maps their positions.
///
/// # Returns
/// * bool - Whether all non-zero expected UTXOs are found
/// * [b256; 5] - UTXOs in order from expected_utxos
/// * [u64; 5] - Index of each UTXO in tx_inputs (u64::max() if zero/not found)
pub fn check_utxos(
    tx_inputs: Vec<InpOut>,
    expected_utxos: [b256; 5],
) -> (bool, [b256; 5], [u64; 5]) {
    let mut ordered_utxos = [b256::zero(); 5];
    let mut utxo_indices = [u64::max(); 5];
    let mut all_found = true;

    // For each expected UTXO
    let mut i = 0;
    while i < 5 {
        let expected_utxo = expected_utxos[i];
        ordered_utxos[i] = expected_utxo;

        // Skip if zero
        if expected_utxo == b256::zero() {
            i += 1;
            continue;
        }

        // Search in tx_inputs
        let mut found = false;
        let mut j = 0;
        while j < tx_inputs.len() {
            // Check if this input has a UTXO and if it matches
            if let Some(tx_utxo) = tx_inputs.get(j).unwrap().uxtoid {
                if tx_utxo == expected_utxo {
                    utxo_indices[i] = j;
                    found = true;
                    break;
                }
            }
            j += 1;
        }

        if !found {
            all_found = false;
        }

        i += 1;
    }

    (all_found, ordered_utxos, utxo_indices)
}


/// Verifies that there exists a change output for the input asset that matches the sender.
///
/// # Arguments
///
/// * `tx_change_assetid`: A Vec of b256 values representing the change output asset IDs.
/// * `tx_change_to`: A Vec of b256 values representing the change output receivers.
/// * `expected_asset_in`: A b256 value representing the expected input asset ID.
/// * `sender`: A b256 value representing the expected sender's address.
///
/// # Returns
///
/// * `bool`: Returns `true` if a matching change output is found for the input asset, `false` otherwise.
/// * The asset and sender must match at the same index in their respective arrays.
///
pub fn verify_change_output(
    tx_change_assetid: Vec<b256>,
    tx_change_to: Vec<b256>,
    expected_asset_in: b256,
    sender: b256,
) -> bool {
    // Skip check if input asset is zero
    if expected_asset_in == b256::zero() {
        return true;
    }

    // Ensure arrays are same length
    if tx_change_assetid.len() != tx_change_to.len() {
        return false;
    }

    // Check each index for matching asset and sender
    let mut i = 0;
    while i < tx_change_assetid.len() {
        if tx_change_assetid.get(i).unwrap() == expected_asset_in &&
           tx_change_to.get(i).unwrap() == sender {
            return true;
        }
        i += 1;
    }

    false
}


struct AssetProcessingResult {
    pub agg_assets: [b256; 3],
    pub agg_amounts: [b256; 3],
    pub match_asset: b256,
    pub match_count: u64,
    pub all_same_type: bool,
    pub amounts_match: bool,
}

pub fn process_assets(
    tx_assets: Vec<InpOut>,
    expected_assets: [b256; 5],
    expected_amounts: [b256; 5],
    expected_output_asset: b256,
    expected_output_amount: b256,
    tolerance_bps: u64,
    is_input: bool
) -> Result<AssetProcessingResult, u64> {
    match aggregate_assets(tx_assets) {
        AggregateResult::Success((agg_assets, agg_amounts)) => {
            match compare_assets_simplified(agg_assets, expected_assets, expected_output_asset) {
                CompareResult::Success((match_asset, match_count, out_match, out_count, all_same_type)) => {
                    let amounts_match = if is_input {
                        compare_input_amounts(agg_assets, agg_amounts, expected_assets, expected_amounts)
                    } else {
                        compare_output_amounts(
                            agg_assets, agg_amounts,
                            expected_output_asset, expected_output_amount,
                            expected_assets, tolerance_bps
                        )
                    };

                    Ok(AssetProcessingResult {
                        agg_assets,
                        agg_amounts,
                        match_asset: if is_input { match_asset } else { out_match },
                        match_count: if is_input { match_count } else { out_count },
                        all_same_type,
                        amounts_match,
                    })
                },
                CompareResult::Fail(error_code) => Err(error_code),
            }
        },
        AggregateResult::Fail(error_code) => Err(error_code),
    }
}



struct ReconstructedIntent {
    pub input_assets: [b256; 5],
    pub input_amounts: [b256; 5],
    pub input_utxos: [b256; 5],    // Added field for UTXOs
    pub output_asset: b256,
    pub output_amount: b256,
}

pub enum ReconstructIntentResult {
    Success: ReconstructedIntent,
    Fail: u64,
}

/// Reconstructs an intent from transaction inputs and aggregated data, preserving UTXO ordering
///
/// # Arguments
/// * ordered_utxos - Array of UTXOs in their intended order
/// * utxo_indices - Array of indices where UTXOs appear in tx_inputs
pub fn reconstruct_intent(
    tx_inputs: Vec<InpOut>,
    agg_input_assets: [b256; 3],
    agg_input_amounts: [b256; 3],
    agg_output_assets: [b256; 3],
    agg_output_amounts: [b256; 3],
    in_match: b256,
    in_count: u64,
    out_match: b256,
    out_count: u64,
    all_same_type: bool,
    ordered_utxos: [b256; 5],
    utxo_indices: [u64; 5],
) -> ReconstructIntentResult {
    if !all_same_type {
        return ReconstructIntentResult::Fail(1);
    }

    let mut intent = ReconstructedIntent {
        input_assets: [b256::zero(); 5],
        input_amounts: [b256::zero(); 5],
        input_utxos: [b256::zero(); 5],
        output_asset: b256::zero(),
        output_amount: b256::zero(),
    };

    // Copy UTXOs in their original order
    let mut i = 0;
    while i < 5 {
        intent.input_utxos[i] = ordered_utxos[i];
        i += 1;
    }

    // Track matched inputs
    let mut matched_count: u64 = 0;

    // Process inputs according to UTXO ordering
    let mut i = 0;
    while i < 5 {
        // Skip zero UTXOs
        if ordered_utxos[i] == b256::zero() {
            i += 1;
            continue;
        }

        // Get the index in tx_inputs
        let tx_index = utxo_indices[i];
        if tx_index == u64::max() {
            return ReconstructIntentResult::Fail(2);
        }

        // Get the input at this index
        let input = tx_inputs.get(tx_index).unwrap();

        // Verify asset type matches
        if input.assetid != in_match {
            return ReconstructIntentResult::Fail(5);
        }

        // Set asset and amount in the same order as UTXOs
        intent.input_assets[i] = input.assetid;
        intent.input_amounts[i] = match input.amount {
            Some(amount) => to_b256((0, 0, 0, amount)),
            None => match input.amount32 {
                Some(amount32) => amount32,
                None => return ReconstructIntentResult::Fail(2),
            },
        };

        matched_count += 1;
        i += 1;
    }

    // Verify match count
    if matched_count != in_count {
        return ReconstructIntentResult::Fail(3);
    }

    // Handle output
    if out_count == 1 {
        intent.output_asset = out_match;
        let mut i = 0;
        while i < 3 {
            if agg_output_assets[i] == out_match {
                intent.output_amount = agg_output_amounts[i];
                break;
            }
            i += 1;
        }
    }

    if intent.output_asset == b256::zero() && out_count == 1 {
        return ReconstructIntentResult::Fail(4);
    }

    ReconstructIntentResult::Success(intent)
}

/// This is a "lite" version of the above, for the predicate validator.
/// The predicate validator has run into max bytecode length, so we are
/// simply reconstructing the intent with only the tx_inputs.
pub fn reconstruct_intent_lite(
    tx_inputs: Vec<InpOut>,
    agg_input_assets: [b256; 3],
    input_amounts: [b256; 5],
    output_asset: b256,
    output_amount: b256,
    ordered_utxos: [b256; 5],
    utxo_indices: [u64; 5],
) -> ReconstructIntentResult {

    let mut intent = ReconstructedIntent {
        input_assets: [b256::zero(); 5],
        input_amounts: input_amounts,
        input_utxos: ordered_utxos,
        output_asset: output_asset,
        output_amount: output_amount,
    };


    let mut i = 0;

    while i < 5 {
        // Skip zero UTXOs
        if ordered_utxos[i] == b256::zero() {
            i += 1;
            continue;
        }

        // Get the index in tx_inputs
        let tx_index = utxo_indices[i];

        // Get the input at this index
        let input = tx_inputs.get(tx_index).unwrap();

        // Set asset and amount in the same order as UTXOs
        intent.input_assets[i] = input.assetid;

        i += 1;
    }

    ReconstructIntentResult::Success(intent)
}







// Helper function to check if an array of 5 elements contains a value
fn contains_val_5(array: [b256; 5], value: b256) -> bool {
    let mut i = 0;
    while i < 5 {
        if array[i] == value {
            return true;
        }
        i += 1;
    }
    false
}






