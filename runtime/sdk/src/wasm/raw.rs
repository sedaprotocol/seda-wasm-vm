#[link(wasm_import_module = "seda_v1")]
extern "C" {
    pub fn shared_memory_read(key: *const u8, key_length: i64, result_data_ptr: *const u8, result_data_length: i64);
    pub fn shared_memory_contains_key(key: *const u8, key_length: i64) -> u8;
    pub fn shared_memory_read_length(key: *const u8, key_length: i64) -> i64;
    pub fn shared_memory_remove(key: *const u8, key_length: i64);
    pub fn shared_memory_range(from: *const u8, from_length: u32, to: *const u8, to_length: u32) -> u32;
    pub fn shared_memory_write(key: *const u8, key_length: i64, value: *const u8, value_length: i64);
    pub fn execution_result(result: *const u8, result_length: i32);

    // Call actions
    pub fn http_fetch(action: *const u8, action_length: u32) -> u32;
    pub fn chain_view(action: *const u8, action_length: u32) -> u32;
    pub fn chain_send_tx(action: *const u8, action_length: u32) -> u32;
    pub fn chain_tx_status(action: *const u8, action_length: u32) -> u32;
    pub fn main_chain_call(action: *const u8, action_length: u32) -> u32;
    pub fn main_chain_view(action: *const u8, action_length: u32) -> u32;
    pub fn main_chain_query(action: *const u8, action_length: u32) -> u32;
    pub fn vm_call(action: *const u8, action_length: u32) -> u32;
    pub fn db_get(action: *const u8, action_length: u32) -> u32;
    pub fn db_set(action: *const u8, action_length: u32) -> u32;
    pub fn p2p_broadcast(action: *const u8, action_length: u32);
    pub fn trigger_event(action: *const u8, action_length: u32);

    // Wasm Storage
    pub fn wasm_exists(action: *const u8, action_length: u32) -> u8;
    pub fn wasm_store(action: *const u8, action_length: u32) -> u32;

    // Reading call actions result
    pub fn call_result_write(result: *const u8, result_length: u32);

    pub fn _log(
        level: *const u8,
        level_len: i32,
        msg: *const u8,
        msg_len: i64,
        line_info: *const u8,
        line_info_len: i64,
    );
    pub fn bn254_verify(
        message: *const u8,
        message_length: i64,
        signature: *const u8,
        signature_length: i64,
        public_key: *const u8,
        public_key_length: i64,
    ) -> u8;
}
