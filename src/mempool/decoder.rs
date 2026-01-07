use alloy::consensus::Transaction as TransactionTrait; 
use alloy::primitives::{Address, Bytes, U256, address};
use alloy::rpc::types::Transaction;

pub const UNISWAP_V2_ROUTER: Address = address!("eE567Fe1712Faf6149d80dA1E6934E354124CfE3");
pub const UNISWAP_V3_ROUTER: Address = address!("3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E"); 
pub const UNISWAP_UNIVERSAL: Address = address!("3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD");
pub const UNISWAP_UNIVERSAL_2: Address = address!("5E325eDA8064b456f4781070C0738d849c824258");

#[derive(Debug, Clone)]
pub struct DecodedSwap {
    #[allow(dead_code)]
    pub router: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub method_name: &'static str,
}

pub struct TransactionDecoder;

impl TransactionDecoder {
    #[inline(always)]
    pub fn is_dex_swap(tx: &Transaction) -> bool {
        let Some(to) = tx.inner.to() else { return false };
        to == UNISWAP_V3_ROUTER || to == UNISWAP_V2_ROUTER || to == UNISWAP_UNIVERSAL || to == UNISWAP_UNIVERSAL_2
    }

    pub fn decode_swap(tx: &Transaction) -> Option<DecodedSwap> {
        let input = tx.inner.input();
        if input.len() < 4 { return None; }
        let to = tx.inner.to()?;
        let selector = &input[0..4];
        
        match selector {
            [0x04, 0xe4, 0x5a, 0xaf] => Self::decode_v3_exact_input_single_new(input, tx, to),
            [0x50, 0x23, 0xb4, 0xdf] => Self::decode_v3_exact_output_single_new(input, to),
            [0x41, 0x4b, 0xf3, 0x89] => Self::decode_v3_exact_input_single_old(input, to),
            [0x38, 0xed, 0x17, 0x39] => Self::decode_v2_swap_exact_tokens_for_tokens(input, to),
            _ => None
        }
    }

    fn decode_v3_exact_input_single_new(input: &Bytes, tx: &Transaction, router: Address) -> Option<DecodedSwap> {
        if input.len() < 196 { return None; }
        let data = &input[4..];
        let token_in = Address::from_slice(&data[12..32]);
        let token_out = Address::from_slice(&data[44..64]);
        let amount_in = U256::from_be_slice(&data[128..160]);
        let final_amount_in = if amount_in.is_zero() { tx.inner.value() } else { amount_in };

        Some(DecodedSwap { router, token_in, token_out, amount_in: final_amount_in, method_name: "exactInputSingle" })
    }

    fn decode_v3_exact_output_single_new(input: &Bytes, router: Address) -> Option<DecodedSwap> {
        if input.len() < 196 { return None; }
        let data = &input[4..];
        let token_in = Address::from_slice(&data[12..32]);
        let token_out = Address::from_slice(&data[44..64]);
        let amount_in_max = U256::from_be_slice(&data[160..192]);
        Some(DecodedSwap { router, token_in, token_out, amount_in: amount_in_max, method_name: "exactOutputSingle" })
    }

    fn decode_v3_exact_input_single_old(input: &Bytes, router: Address) -> Option<DecodedSwap> {
        if input.len() < 164 { return None; }
        let data = &input[4..];
        let token_in = Address::from_slice(&data[12..32]);
        let token_out = Address::from_slice(&data[44..64]);
        let amount_in = U256::from_be_slice(&data[160..192]);
        Some(DecodedSwap { router, token_in, token_out, amount_in, method_name: "exactInputSingle(old)" })
    }

    fn decode_v2_swap_exact_tokens_for_tokens(input: &Bytes, router: Address) -> Option<DecodedSwap> {
        if input.len() < 100 { return None; }
        let data = &input[4..];
        let amount_in = U256::from_be_slice(&data[0..32]);
        Some(DecodedSwap { router, token_in: Address::ZERO, token_out: Address::ZERO, amount_in, method_name: "swapExactTokensForTokens(V2)" })
    }
}