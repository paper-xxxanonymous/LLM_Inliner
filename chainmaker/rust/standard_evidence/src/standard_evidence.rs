use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use contract_sdk_rust::easycodec::*;

const PARAM_ID: &str = "id";
const PARAM_HASH: &str = "hash";
const PARAM_METADATA: &str = "metadata";
const PARAM_EVIDENCES: &str = "evidences";
const PARAM_STANDARD_NAME: &str = "standardName";

const METHOD_PRE_EVIDENCE: &str = "[evidence]";
const METHOD_PRE_EXISTS_OF_HASH: &str = "[existsOfHash]";
const METHOD_PRE_EXISTS_OF_ID: &str = "[existsOfId]";
const METHOD_PRE_FIND_BY_HASH: &str = "[findByHash]";
const METHOD_PRE_FIND_BY_ID: &str = "[findById]";
const METHOD_PRE_EVIDENCE_BATCH: &str = "[evidenceBatch]";
const METHOD_PRE_STANDARDS: &str = "[standards]";
const METHOD_PRE_SUPPORTS_STANDARD: &str = "[supportsStandard]";

const KEY_EVIDENCE_HASH: &str = "h";
const KEY_EVIDENCE_ID: &str = "i";
const TRUE_STRING: &str = "1";
const FALSE_STRING: &str = "0";

// 合约标准名称
const CONTRACT_STANDARD_NAME_CMEVI: &str = "CMEVI";
const CONTRACT_STANDARD_NAME_CMBC: &str = "CMBC";

// 存证对象
struct Evidence {
    id: String,
    tx_id: String,
    hash: String,
    block_height: i32,
    metadata: String,
    ec: EasyCodec,
}

impl Evidence {
    fn new_evidence(id: String, tx_id: String, hash: String, block_height: i32, metadata: String) -> Evidence {
        let mut ec = EasyCodec::new();
        ec.add_string("id", id.as_str());
        ec.add_string("txId", tx_id.as_str());
        ec.add_string("hash", hash.as_str());
        ec.add_i32("blockHeight", block_height);
        ec.add_string("metadata", metadata.as_str());
        Evidence {
            id,
            tx_id,
            hash,
            block_height,
            metadata,
            ec,
        }
    }

    fn get_emit_event_data(&self) -> Vec<String> {
        let mut arr: Vec<String> = Vec::new();
        arr.push(self.id.clone());
        arr.push(self.tx_id.clone());
        arr.push(self.hash.clone());
        arr.push(self.block_height.to_string());
        arr.push(self.metadata.clone());
        arr
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Evidence {
        let ec = EasyCodec::new_with_bytes(data);
        Evidence {
            id: ec.get_string("id").unwrap_or("".to_string()),
            tx_id: ec.get_string("txId").unwrap_or("".to_string()),
            hash: ec.get_string("hash").unwrap_or("".to_string()),
            block_height: ec.get_i32("blockHeight").unwrap_or(0),
            metadata: ec.get_string("metadata").unwrap_or("".to_string()),
            ec,
        }
    }
}

// 合约安装函数
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Init contract success".as_bytes());
}

// 合约更新函数
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}

// 核心业务函数 - 存证
#[no_mangle]
pub extern "C" fn evidence() {
    let ctx = &mut sim_context::get_sim_context();

    // 获取参数
    let id = ctx.arg_as_utf8_str(PARAM_ID);
    let hash = ctx.arg_as_utf8_str(PARAM_HASH);
    let metadata = ctx.arg_as_utf8_str(PARAM_METADATA);

    match do_evidence(id, hash, metadata) {
        Ok(_) => {
            ctx.ok("evidence saved".as_bytes());
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EVIDENCE, e);
            ctx.error(&err_msg);
        }
    }
}

// 批量存证
#[no_mangle]
pub extern "C" fn evidence_batch() {
    let ctx = &mut sim_context::get_sim_context();
    let evidences_param = ctx.arg_as_utf8_str(PARAM_EVIDENCES);

    if evidences_param.is_empty() {
        let err_msg = format!("{}evidences is empty", METHOD_PRE_EVIDENCE_BATCH);
        ctx.error(&err_msg);
        return;
    }

    let evidences = match parse_evidences_json(&evidences_param) {
        Ok(ev) => ev,
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EVIDENCE_BATCH, e);
            ctx.error(&err_msg);
            return;
        }
    };

    match do_evidence_batch(evidences) {
        Ok(_) => {
            ctx.ok("ok".as_bytes());
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EVIDENCE_BATCH, e);
            ctx.error(&err_msg);
        }
    }
}

// 根据哈希查询是否存在
#[no_mangle]
pub extern "C" fn exists_of_hash() {
    let ctx = &mut sim_context::get_sim_context();
    let hash = ctx.arg_as_utf8_str(PARAM_HASH);

    if hash.is_empty() {
        let err_msg = format!("{}hash cannot be empty", METHOD_PRE_EXISTS_OF_HASH);
        ctx.error(&err_msg);
        return;
    }

    match do_exists_of_hash(&hash) {
        Ok(exists) => {
            if exists {
                ctx.ok(TRUE_STRING.as_bytes());
            } else {
                ctx.ok(FALSE_STRING.as_bytes());
            }
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EXISTS_OF_HASH, e);
            ctx.error(&err_msg);
        }
    }
}

// 根据ID查询是否存在
#[no_mangle]
pub extern "C" fn exists_of_id() {
    let ctx = &mut sim_context::get_sim_context();
    let id = ctx.arg_as_utf8_str(PARAM_ID);

    if id.is_empty() {
        let err_msg = format!("{}id cannot be empty", METHOD_PRE_EXISTS_OF_ID);
        ctx.error(&err_msg);
        return;
    }

    match do_exists_of_id(&id) {
        Ok(exists) => {
            if exists {
                ctx.ok(TRUE_STRING.as_bytes());
            } else {
                ctx.ok(FALSE_STRING.as_bytes());
            }
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EXISTS_OF_ID, e);
            ctx.error(&err_msg);
        }
    }
}

// 根据哈希查找
#[no_mangle]
pub extern "C" fn find_by_hash() {
    let ctx = &mut sim_context::get_sim_context();
    let hash = ctx.arg_as_utf8_str(PARAM_HASH);

    if hash.is_empty() {
        ctx.error("hash cannot be empty");
        return;
    }

    match do_find_by_hash(&hash) {
        Ok(evidence) => {
            let json_str = evidence.to_json();
            ctx.ok(json_str.as_bytes());
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_FIND_BY_HASH, e);
            ctx.error(&err_msg);
        }
    }
}

// 存证并查询
#[no_mangle]
pub extern "C" fn evidenceAndFindByHash() {
    let ctx = &mut sim_context::get_sim_context();
    let evidences_param = ctx.arg_as_utf8_str(PARAM_EVIDENCES);

    if evidences_param.is_empty() {
        let err_msg = format!("{}evidences is empty", METHOD_PRE_EVIDENCE_BATCH);
        ctx.error(&err_msg);
        return;
    }

    let evidences = match parse_evidences_json(&evidences_param) {
        Ok(ev) => ev,
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EVIDENCE_BATCH, e);
            ctx.error(&err_msg);
            return;
        }
    };

    match do_evidence_batch(evidences) {
        Ok(_) => {
            // 存证成功，继续查询
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_EVIDENCE_BATCH, e);
            ctx.error(&err_msg);
            return;
        }
    };

    let hash = ctx.arg_as_utf8_str(PARAM_HASH);
    if hash.is_empty() {
        ctx.error("hash cannot be empty");
        return;
    }

    match do_find_by_hash(&hash) {
        Ok(evidence) => {
            let json_str = evidence.to_json();
            ctx.ok(json_str.as_bytes());
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_FIND_BY_HASH, e);
            ctx.error(&err_msg);
        }
    }
}

// 根据ID查找
#[no_mangle]
pub extern "C" fn find_by_id() {
    let ctx = &mut sim_context::get_sim_context();
    let id = ctx.arg_as_utf8_str(PARAM_ID);

    if id.is_empty() {
        let err_msg = format!("{}id cannot be empty", METHOD_PRE_FIND_BY_ID);
        ctx.error(&err_msg);
        return;
    }

    match do_find_by_id(&id) {
        Ok(evidence) => {
            let json_str = evidence.to_json();
            ctx.ok(json_str.as_bytes());
        }
        Err(e) => {
            let err_msg = format!("{}{}", METHOD_PRE_FIND_BY_ID, e);
            ctx.error(&err_msg);
        }
    }
}

// 获取支持的标准列表
#[no_mangle]
pub extern "C" fn standards() {
    let ctx = &mut sim_context::get_sim_context();
    let standards_list = do_standards();
    let json_str = standards_list_to_json(&standards_list);
    ctx.ok(json_str.as_bytes());
}

// 判断是否支持某个标准
#[no_mangle]
pub extern "C" fn support_standard() {
    let ctx = &mut sim_context::get_sim_context();
    let standard_name = ctx.arg_as_utf8_str(PARAM_STANDARD_NAME);

    if standard_name.is_empty() {
        let err_msg = format!("{}standardName cannot be empty", METHOD_PRE_SUPPORTS_STANDARD);
        ctx.error(&err_msg);
        return;
    }

    if do_support_standard(&standard_name) {
        ctx.ok(TRUE_STRING.as_bytes());
    } else {
        ctx.ok(FALSE_STRING.as_bytes());
    }
}

// ========== 内部函数 ==========

// 存证核心逻辑
fn do_evidence(id: String, hash: String, metadata: String) -> Result<(), String> {
    let ctx = &mut sim_context::get_sim_context();

    // 获取区块信息
    let tx_id = ctx.get_tx_id();
    let block_height = ctx.get_block_height();

    // 构建存证对象
    let evidence = Evidence::new_evidence(id, tx_id, hash.clone(), block_height as i32, metadata);

    let data = evidence.marshal();

    // 存储数据
    let code = ctx.put_state(KEY_EVIDENCE_HASH, &hash, data.as_slice());
    if code != 0 {
        return Err("put evidence hash failed".to_string());
    }

    let code = ctx.put_state(KEY_EVIDENCE_ID, &evidence.id, hash.as_bytes());
    if code != 0 {
        return Err("put evidence id failed".to_string());
    }

    // 发送事件
    ctx.emit_event("evidence", &evidence.get_emit_event_data());

    Ok(())
}

// 批量存证
fn do_evidence_batch(evidences: Vec<Evidence>) -> Result<(), String> {
    for evidence in evidences {
        do_evidence(evidence.id, evidence.hash, evidence.metadata)?;
    }
    Ok(())
}

// 根据哈希查询是否存在
fn do_exists_of_hash(hash: &str) -> Result<bool, String> {
    let ctx = &mut sim_context::get_sim_context();
    let result = ctx.get_state(KEY_EVIDENCE_HASH, hash);
    match result {
        Ok(data) => Ok(data.len() > 0),
        Err(_) => Ok(false),
    }
}

// 根据ID查询是否存在
fn do_exists_of_id(id: &str) -> Result<bool, String> {
    let ctx = &mut sim_context::get_sim_context();
    let result = ctx.get_state(KEY_EVIDENCE_ID, id);
    match result {
        Ok(data) => Ok(data.len() > 0),
        Err(_) => Ok(false),
    }
}

// 根据哈希查找
fn do_find_by_hash(hash: &str) -> Result<Evidence, String> {
    let ctx = &mut sim_context::get_sim_context();
    let data = match ctx.get_state(KEY_EVIDENCE_HASH, hash) {
        Ok(d) => d,
        Err(e) => return Err("get evidence failed".to_string()),
    };

    if data.is_empty() {
        return Err("evidence not found".to_string());
    }

    Ok(Evidence::unmarshal(&data))
}

// 根据ID查找
fn do_find_by_id(id: &str) -> Result<Evidence, String> {
    let ctx = &mut sim_context::get_sim_context();
    let hash = match ctx.get_state(KEY_EVIDENCE_ID, id) {
        Ok(h) => h,
        Err(e) => return Err("get evidence id failed".to_string()),
    };

    let hash_str = match String::from_utf8(hash) {
        Ok(s) => s,
        Err(e) => return Err("invalid hash data".to_string()),
    };

    do_find_by_hash(&hash_str)
}

// 获取支持的标准列表
fn do_standards() -> Vec<String> {
    vec![
        CONTRACT_STANDARD_NAME_CMEVI.to_string(),
        CONTRACT_STANDARD_NAME_CMBC.to_string(),
    ]
}

// 判断是否支持某个标准
fn do_support_standard(standard_name: &str) -> bool {
    standard_name == CONTRACT_STANDARD_NAME_CMEVI || standard_name == CONTRACT_STANDARD_NAME_CMBC
}

// 解析存证JSON数组
// 期望格式: [{"id":"1","hash":"abc","metadata":"xxx"}, ...]
fn parse_evidences_json(json_str: &str) -> Result<Vec<Evidence>, String> {
    let mut evidences = Vec::new();

    // 查找数组的开始和结束
    let start = match json_str.find('[') {
        Some(s) => s + 1,
        None => return Err("invalid JSON: missing array start".to_string()),
    };
    let end = match json_str.rfind(']') {
        Some(e) => e,
        None => return Err("invalid JSON: missing array end".to_string()),
    };

    if start >= end {
        return Ok(Vec::new());
    }

    let array_content = &json_str[start..end];
    let mut pos = 0;

    while pos < array_content.len() {
        // 查找对象开始
        let obj_start = match array_content[pos..].find('{') {
            Some(s) => s + pos,
            None => break,
        };

        // 查找对应的对象结束
        let mut brace_count = 0;
        let mut obj_end = obj_start;
        let chars: Vec<char> = array_content.chars().collect();
        for i in obj_start..chars.len() {
            if chars[i] == '{' {
                brace_count += 1;
            } else if chars[i] == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    obj_end = i;
                    break;
                }
            }
        }

        if obj_end <= obj_start {
            break;
        }

        let obj_str = &array_content[obj_start..=obj_end];

        // 解析 id
        let id = match parse_json_string(obj_str, "id") {
            Some(v) => v,
            None => return Err("invalid evidence: missing id".to_string()),
        };

        // 解析 hash
        let hash = match parse_json_string(obj_str, "hash") {
            Some(v) => v,
            None => return Err("invalid evidence: missing hash".to_string()),
        };

        // 解析 metadata
        let metadata = match parse_json_string(obj_str, "metadata") {
            Some(v) => v,
            None => "".to_string(),
        };

        let evidence = Evidence::new_evidence(id, "".to_string(), hash, 0, metadata);
        evidences.push(evidence);

        pos = obj_end + 1;
    }

    Ok(evidences)
}

// 从JSON对象字符串中解析指定字段的值
fn parse_json_string(obj_str: &str, field: &str) -> Option<String> {
    let search_str = format!("\"{}\":\"", field);
    let start = match obj_str.find(&search_str) {
        Some(s) => s + search_str.len(),
        None => return None,
    };

    let end = match obj_str[start..].find('\"') {
        Some(e) => start + e,
        None => return None,
    };

    Some(obj_str[start..end].to_string())
}

// 将标准列表转换为JSON字符串
fn standards_list_to_json(list: &Vec<String>) -> String {
    let mut json = String::from("[");
    for (i, item) in list.iter().enumerate() {
        if i > 0 {
            json.push_str(",");
        }
        json.push_str("\"");
        json.push_str(item);
        json.push_str("\"");
    }
    json.push_str("]");
    json
}

// 简单的字符串工具函数
fn is_any_blank(s: &str) -> bool {
    s.is_empty() || s.trim().is_empty()
}