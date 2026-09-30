use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::{SimContext, HistoryKvIter};
use contract_sdk_rust::easycodec::*;

// ========== 常量定义 ==========


// ========== 数据结构定义 ==========

// Asn 数据结构
struct Asn {
    asn: String,
    name: String,
    domain: String,
    route: String,
    r#type: String,
    ec: EasyCodec,
}

impl Asn {
    fn new_asn(asn: String, name: String, domain: String, route: String, r#type: String) -> Asn {
        let mut ec = EasyCodec::new();
        ec.add_string("asn", &asn);
        ec.add_string("name", &name);
        ec.add_string("domain", &domain);
        ec.add_string("route", &route);
        ec.add_string("type", &r#type);
        Asn { asn, name, domain, route, r#type, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Asn {
        let ec = EasyCodec::new_with_bytes(data);
        Asn {
            asn: ec.get_string("asn").unwrap_or("".to_string()),
            name: ec.get_string("name").unwrap_or("".to_string()),
            domain: ec.get_string("domain").unwrap_or("".to_string()),
            route: ec.get_string("route").unwrap_or("".to_string()),
            r#type: ec.get_string("type").unwrap_or("".to_string()),
            ec,
        }
    }
}

// Company 数据结构
struct Company {
    name: String,
    domain: String,
    r#type: String,
    ec: EasyCodec,
}

impl Company {
    fn new_company(name: String, domain: String, r#type: String) -> Company {
        let mut ec = EasyCodec::new();
        ec.add_string("name", &name);
        ec.add_string("domain", &domain);
        ec.add_string("type", &r#type);
        Company { name, domain, r#type, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Company {
        let ec = EasyCodec::new_with_bytes(data);
        Company {
            name: ec.get_string("name").unwrap_or("".to_string()),
            domain: ec.get_string("domain").unwrap_or("".to_string()),
            r#type: ec.get_string("type").unwrap_or("".to_string()),
            ec,
        }
    }
}

// Privacy 数据结构
struct Privacy {
    vpn: bool,
    proxy: bool,
    tor: bool,
    relay: bool,
    hosting: bool,
    service: String,
    ec: EasyCodec,
}

impl Privacy {
    fn new_privacy(vpn: bool, proxy: bool, tor: bool, relay: bool, hosting: bool, service: String) -> Privacy {
        let mut ec = EasyCodec::new();
        ec.add_string("vpn", if vpn { "true" } else { "false" });
        ec.add_string("proxy", if proxy { "true" } else { "false" });
        ec.add_string("tor", if tor { "true" } else { "false" });
        ec.add_string("relay", if relay { "true" } else { "false" });
        ec.add_string("hosting", if hosting { "true" } else { "false" });
        ec.add_string("service", &service);
        Privacy { vpn, proxy, tor, relay, hosting, service, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Privacy {
        let ec = EasyCodec::new_with_bytes(data);
        Privacy {
            vpn: ec.get_string("vpn").unwrap_or("false".to_string()) == "true",
            proxy: ec.get_string("proxy").unwrap_or("false".to_string()) == "true",
            tor: ec.get_string("tor").unwrap_or("false".to_string()) == "true",
            relay: ec.get_string("relay").unwrap_or("false".to_string()) == "true",
            hosting: ec.get_string("hosting").unwrap_or("false".to_string()) == "true",
            service: ec.get_string("service").unwrap_or("".to_string()),
            ec,
        }
    }
}

// Abuse 数据结构
struct Abuse {
    address: String,
    country: String,
    email: String,
    name: String,
    network: String,
    phone: String,
    ec: EasyCodec,
}

impl Abuse {
    fn new_abuse(address: String, country: String, email: String, name: String, network: String, phone: String) -> Abuse {
        let mut ec = EasyCodec::new();
        ec.add_string("address", &address);
        ec.add_string("country", &country);
        ec.add_string("email", &email);
        ec.add_string("name", &name);
        ec.add_string("network", &network);
        ec.add_string("phone", &phone);
        Abuse { address, country, email, name, network, phone, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Abuse {
        let ec = EasyCodec::new_with_bytes(data);
        Abuse {
            address: ec.get_string("address").unwrap_or("".to_string()),
            country: ec.get_string("country").unwrap_or("".to_string()),
            email: ec.get_string("email").unwrap_or("".to_string()),
            name: ec.get_string("name").unwrap_or("".to_string()),
            network: ec.get_string("network").unwrap_or("".to_string()),
            phone: ec.get_string("phone").unwrap_or("".to_string()),
            ec,
        }
    }
}

// Domains 数据结构
struct Domains {
    total: i32,
    domains: Vec<String>,
    ec: EasyCodec,
}

impl Domains {
    fn new_domains(total: i32, domains: Vec<String>) -> Domains {
        let mut ec = EasyCodec::new();
        ec.add_i32("total", total);
        for (i, domain) in domains.iter().enumerate() {
            let key = format!("domain_{}", i);
            ec.add_string(&key, domain);
        }
        Domains { total, domains, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Domains {
        let ec = EasyCodec::new_with_bytes(data);
        let total = ec.get_i32("total").unwrap_or(0);
        let mut domains = Vec::new();
        for i in 0..total {
            let key = format!("domain_{}", i);
            let domain = ec.get_string(&key).unwrap_or("".to_string());
            if !domain.is_empty() {
                domains.push(domain);
            }
        }
        Domains { total, domains, ec }
    }
}

// Itinerary 行程数据结构
struct Itinerary {
    ip: String,
    city: String,
    region: String,
    country: String,
    loc: String,
    org: String,
    timezone: String,
    asn: Asn,
    company: Company,
    privacy: Privacy,
    abuse: Abuse,
    domains: Domains,
    ec: EasyCodec,
}

impl Itinerary {
    fn new_itinerary(
        ip: String,
        city: String,
        region: String,
        country: String,
        loc: String,
        org: String,
        timezone: String,
        asn: Asn,
        company: Company,
        privacy: Privacy,
        abuse: Abuse,
        domains: Domains,
    ) -> Itinerary {
        let mut ec = EasyCodec::new();
        ec.add_string("ip", &ip);
        ec.add_string("city", &city);
        ec.add_string("region", &region);
        ec.add_string("country", &country);
        ec.add_string("loc", &loc);
        ec.add_string("org", &org);
        ec.add_string("timezone", &timezone);
        ec.add_bytes("asn", asn.marshal());
        ec.add_bytes("company", company.marshal());
        ec.add_bytes("privacy", privacy.marshal());
        ec.add_bytes("abuse", abuse.marshal());
        ec.add_bytes("domains", domains.marshal());
        Itinerary { ip, city, region, country, loc, org, timezone, asn, company, privacy, abuse, domains, ec }
    }

    fn get_location(&self) -> String {
        format!("{}-{}-{}", self.country, self.city, self.region)
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Itinerary {
        let ec = EasyCodec::new_with_bytes(data);
        let asn_data = ec.get_bytes("asn").unwrap_or(Vec::new());
        let company_data = ec.get_bytes("company").unwrap_or(Vec::new());
        let privacy_data = ec.get_bytes("privacy").unwrap_or(Vec::new());
        let abuse_data = ec.get_bytes("abuse").unwrap_or(Vec::new());
        let domains_data = ec.get_bytes("domains").unwrap_or(Vec::new());

        Itinerary {
            ip: ec.get_string("ip").unwrap_or("".to_string()),
            city: ec.get_string("city").unwrap_or("".to_string()),
            region: ec.get_string("region").unwrap_or("".to_string()),
            country: ec.get_string("country").unwrap_or("".to_string()),
            loc: ec.get_string("loc").unwrap_or("".to_string()),
            org: ec.get_string("org").unwrap_or("".to_string()),
            timezone: ec.get_string("timezone").unwrap_or("".to_string()),
            asn: Asn::unmarshal(&asn_data),
            company: Company::unmarshal(&company_data),
            privacy: Privacy::unmarshal(&privacy_data),
            abuse: Abuse::unmarshal(&abuse_data),
            domains: Domains::unmarshal(&domains_data),
            ec,
        }
    }
}

// HistoryValue 历史记录
struct HistoryValue {
    field: String,
    value: Itinerary,
    tx_id: String,
    timestamp: String,
    block_height: i32,
    key: String,
    ec: EasyCodec,
}

impl HistoryValue {
    fn new_history_value(field: String, value: Itinerary, tx_id: String, timestamp: String, block_height: i32, key: String) -> HistoryValue {
        let mut ec = EasyCodec::new();
        ec.add_string("field", &field);
        ec.add_bytes("value", value.marshal());
        ec.add_string("txId", &tx_id);
        ec.add_string("timestamp", &timestamp);
        ec.add_i32("blockHeight", block_height);
        ec.add_string("key", &key);
        HistoryValue { field, value, tx_id, timestamp, block_height, key, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }
}

// 空字符串检查
fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

// ========== JSON 解析函数 ==========

// 解析 JSON 字符串值
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

// 解析 JSON 布尔值
fn parse_json_bool(obj_str: &str, field: &str) -> bool {
    let search_str = format!("\"{}\":", field);
    let start = match obj_str.find(&search_str) {
        Some(s) => s + search_str.len(),
        None => return false,
    };
    let end = match obj_str[start..].find(',') {
        Some(e) => start + e,
        None => obj_str.len(),
    };
    let val = obj_str[start..end].trim();
    val == "true" || val == "1"
}

// 解析嵌套对象
fn parse_nested_object(json_str: &str, field: &str) -> Result<String, String> {
    let search_str = format!("\"{}\":", field);
    let start = match json_str.find(&search_str) {
        Some(s) => s + search_str.len(),
        None => return Err(format!("field {} not found", field)),
    };
    let mut pos = start;
    let chars: Vec<char> = json_str.chars().collect();
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    if pos >= chars.len() || chars[pos] != '{' {
        return Err(format!("field {} is not an object", field));
    }
    let mut brace_count = 0;
    let mut end = pos;
    for i in pos..chars.len() {
        if chars[i] == '{' {
            brace_count += 1;
        } else if chars[i] == '}' {
            brace_count -= 1;
            if brace_count == 0 {
                end = i;
                break;
            }
        }
    }
    if end <= pos {
        return Err(format!("invalid object for field {}", field));
    }
    Ok(json_str[pos..=end].to_string())
}

// 解析 JSON 数组
fn parse_json_array(obj_str: &str, field: &str) -> Vec<String> {
    let search_str = format!("\"{}\":", field);
    let start = match obj_str.find(&search_str) {
        Some(s) => s + search_str.len(),
        None => return Vec::new(),
    };
    let mut pos = start;
    let chars: Vec<char> = obj_str.chars().collect();
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    if pos >= chars.len() || chars[pos] != '[' {
        return Vec::new();
    }
    pos += 1;
    let mut result = Vec::new();
    while pos < chars.len() {
        while pos < chars.len() && (chars[pos].is_whitespace() || chars[pos] == ',') {
            pos += 1;
        }
        if pos >= chars.len() || chars[pos] == ']' {
            break;
        }
        if chars[pos] == '"' {
            pos += 1;
            let start_str = pos;
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            let end_str = pos;
            result.push(obj_str[start_str..end_str].to_string());
            pos += 1;
        } else {
            pos += 1;
        }
    }
    result
}

// 解析完整的 Itinerary JSON
fn parse_itinerary_from_json(json_str: &str) -> Result<Itinerary, String> {
    let ip = parse_json_string(json_str, "ip").unwrap_or("".to_string());
    let city = parse_json_string(json_str, "city").unwrap_or("".to_string());
    let region = parse_json_string(json_str, "region").unwrap_or("".to_string());
    let country = parse_json_string(json_str, "country").unwrap_or("".to_string());
    let loc = parse_json_string(json_str, "loc").unwrap_or("".to_string());
    let org = parse_json_string(json_str, "org").unwrap_or("".to_string());
    let timezone = parse_json_string(json_str, "timezone").unwrap_or("".to_string());

    let asn_obj = parse_nested_object(json_str, "asn")?;
    let asn = Asn::new_asn(
        parse_json_string(&asn_obj, "asn").unwrap_or("".to_string()),
        parse_json_string(&asn_obj, "name").unwrap_or("".to_string()),
        parse_json_string(&asn_obj, "domain").unwrap_or("".to_string()),
        parse_json_string(&asn_obj, "route").unwrap_or("".to_string()),
        parse_json_string(&asn_obj, "type").unwrap_or("".to_string()),
    );

    let company_obj = parse_nested_object(json_str, "company")?;
    let company = Company::new_company(
        parse_json_string(&company_obj, "name").unwrap_or("".to_string()),
        parse_json_string(&company_obj, "domain").unwrap_or("".to_string()),
        parse_json_string(&company_obj, "type").unwrap_or("".to_string()),
    );

    let privacy_obj = parse_nested_object(json_str, "privacy")?;
    let privacy = Privacy::new_privacy(
        parse_json_bool(&privacy_obj, "vpn"),
        parse_json_bool(&privacy_obj, "proxy"),
        parse_json_bool(&privacy_obj, "tor"),
        parse_json_bool(&privacy_obj, "relay"),
        parse_json_bool(&privacy_obj, "hosting"),
        parse_json_string(&privacy_obj, "service").unwrap_or("".to_string()),
    );

    let abuse_obj = parse_nested_object(json_str, "abuse")?;
    let abuse = Abuse::new_abuse(
        parse_json_string(&abuse_obj, "address").unwrap_or("".to_string()),
        parse_json_string(&abuse_obj, "country").unwrap_or("".to_string()),
        parse_json_string(&abuse_obj, "email").unwrap_or("".to_string()),
        parse_json_string(&abuse_obj, "name").unwrap_or("".to_string()),
        parse_json_string(&abuse_obj, "network").unwrap_or("".to_string()),
        parse_json_string(&abuse_obj, "phone").unwrap_or("".to_string()),
    );

    let domains_obj = parse_nested_object(json_str, "domains")?;
    let total_str = parse_json_string(&domains_obj, "total").unwrap_or("0".to_string());
    let total = total_str.parse::<i32>().unwrap_or(0);
    let domains_list = parse_json_array(&domains_obj, "domains");
    let domains = Domains::new_domains(total, domains_list);

    Ok(Itinerary::new_itinerary(
        ip, city, region, country, loc, org, timezone,
        asn, company, privacy, abuse, domains,
    ))
}

// 解析 HistoryValue 用于更新记录
fn parse_history_value_from_json(json_str: &str) -> Option<HistoryValue> {
    let block_height_str = parse_json_string(json_str, "blockHeight")?;
    let block_height = block_height_str.parse::<i32>().unwrap_or(0);
    let field = parse_json_string(json_str, "field").unwrap_or("".to_string());
    let tx_id = parse_json_string(json_str, "txId").unwrap_or("".to_string());
    let timestamp = parse_json_string(json_str, "timestamp").unwrap_or("".to_string());
    let key = parse_json_string(json_str, "key").unwrap_or("".to_string());

    let value_obj = match parse_nested_object(json_str, "value") {
        Ok(obj) => obj,
        Err(_) => return None,
    };
    let value = match parse_itinerary_from_json(&value_obj) {
        Ok(it) => it,
        Err(_) => return None,
    };

    Some(HistoryValue::new_history_value(field, value, tx_id, timestamp, block_height, key))
}

// ========== 合约函数 ==========

// 安装合约
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Init contract success".as_bytes());
}

// 升级合约
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}

// 保存行程数据
#[no_mangle]
pub extern "C" fn save() {
    let ctx = &mut sim_context::get_sim_context();

    let itinerary_str = ctx.arg_as_utf8_str("itinerary");
    let phone = ctx.arg_as_utf8_str("phone");

    if is_blank(&itinerary_str) {
        ctx.error("'itinerary' should not be empty!");
        return;
    }
    if is_blank(&phone) {
        ctx.error("'phone' should not be empty!");
        return;
    }

    let itinerary = match parse_itinerary_from_json(&itinerary_str) {
        Ok(it) => it,
        Err(e) => {
            let err_msg = format!("unmarshal itinerary data failed, {}", e);
            ctx.error(&err_msg);
            return;
        }
    };

    if is_blank(&itinerary.ip) || is_blank(&itinerary.country) ||
        is_blank(&itinerary.city) || is_blank(&itinerary.region) {
        ctx.error("'ip','country','city','region' should not be empty");
        return;
    }

    let code = ctx.put_state(&phone, "", itinerary_str.as_bytes());
    if code != 0 {
        ctx.error("save data failed");
        return;
    }

    let mut event_data = Vec::new();
    event_data.push(phone);
    event_data.push(itinerary_str);
    ctx.emit_event("save", &event_data);

    // 返回结果: {"code":200,"msg":"","data":<itinerary_json>}
    let result_json = itinerary.to_json();
    let response = format!("{{\"code\":200,\"msg\":\"\",\"data\":{}}}", result_json);
    ctx.ok(response.as_bytes());
}

// 查询历史记录
#[no_mangle]
pub extern "C" fn query_history() {
    let ctx = &mut sim_context::get_sim_context();
    let phone = ctx.arg_as_utf8_str("phone");

    if is_blank(&phone) {
        ctx.error("phone cannot be empty");
        return;
    }

    let iter_result = ctx.new_history_kv_iter_for_key(&phone, "");
    if iter_result.is_err() {
        let err_msg = format!("new HistoryKvIter for key=[{}] failed", phone);
        ctx.error(&err_msg);
        return;
    }
    let iter = iter_result.unwrap();

    // 使用 HashMap 存储最新的记录
    use std::collections::HashMap;
    let mut record_map: HashMap<String, HistoryValue> = HashMap::new();

    while iter.has_next() {
        let km = match iter.next() {
            Ok(k) => k,
            Err(_) => {
                ctx.log("iterator failed to get the next element");
                continue;
            }
        };

        let itinerary = Itinerary::unmarshal(&km.value);
        let location = itinerary.get_location();
        let hv = HistoryValue::new_history_value(
            km.field,
            itinerary,
            km.tx_id,
            km.timestamp,
            km.block_height,
            km.key,
        );

        // 只保留最新的记录（blockHeight 最大的）
        if let Some(existing) = record_map.get(&location) {
            if existing.block_height < km.block_height {
                record_map.insert(location, hv);
            }
        } else {
            record_map.insert(location, hv);
        }
    }

    iter.close();

    // 构建返回的 JSON: {"code":200,"msg":"","data":{...}}
    let mut data_json = String::from("{");
    let mut first = true;
    for (location, hv) in record_map {
        if !first {
            data_json.push_str(",");
        }
        first = false;
        data_json.push_str(&format!("\"{}\":{}", location, hv.to_json()));
    }
    data_json.push_str("}");

    let response = format!("{{\"code\":200,\"msg\":\"\",\"data\":{}}}", data_json);
    ctx.ok(response.as_bytes());
}

// 保存并查询历史
#[no_mangle]
pub extern "C" fn saveAndQueryHistory() {
    let ctx = &mut sim_context::get_sim_context();

    let itinerary_str = ctx.arg_as_utf8_str("itinerary");
    let phone = ctx.arg_as_utf8_str("phone");

    if is_blank(&itinerary_str) {
        ctx.error("'itinerary' should not be empty!");
        return;
    }
    if is_blank(&phone) {
        ctx.error("'phone' should not be empty!");
        return;
    }

    let save_itinerary = match parse_itinerary_from_json(&itinerary_str) {
        Ok(it) => it,
        Err(e) => {
            let err_msg = format!("unmarshal itinerary data failed, {}", e);
            ctx.error(&err_msg);
            return;
        }
    };

    if is_blank(&save_itinerary.ip) || is_blank(&save_itinerary.country) ||
        is_blank(&save_itinerary.city) || is_blank(&save_itinerary.region) {
        ctx.error("'ip','country','city','region' should not be empty");
        return;
    }

    let code = ctx.put_state(&phone, "", itinerary_str.as_bytes());
    if code != 0 {
        ctx.error("save data failed");
        return;
    }

    let mut event_data = Vec::new();
    event_data.push(phone.clone());
    event_data.push(itinerary_str);
    ctx.emit_event("save", &event_data);

    // 查询历史
    let iter_result = ctx.new_history_kv_iter_for_key(&phone, "");
    if iter_result.is_err() {
        let err_msg = format!("new HistoryKvIter for key=[{}] failed", phone);
        ctx.error(&err_msg);
        return;
    }
    let iter = iter_result.unwrap();

    use std::collections::HashMap;
    let mut record_map: HashMap<String, HistoryValue> = HashMap::new();

    while iter.has_next() {
        let km = match iter.next() {
            Ok(k) => k,
            Err(_) => {
                ctx.log("iterator failed to get the next element");
                continue;
            }
        };

        let itinerary = Itinerary::unmarshal(&km.value);
        let location = itinerary.get_location();
        let hv = HistoryValue::new_history_value(
            km.field,
            itinerary,
            km.tx_id,
            km.timestamp,
            km.block_height,
            km.key,
        );

        if let Some(existing) = record_map.get(&location) {
            if existing.block_height < km.block_height {
                record_map.insert(location, hv);
            }
        } else {
            record_map.insert(location, hv);
        }
        // 只取第一条记录（对应 Go 版本的 break）
        break;
    }

    iter.close();

    let mut data_json = String::from("{");
    let mut first = true;
    for (location, hv) in record_map {
        if !first {
            data_json.push_str(",");
        }
        first = false;
        data_json.push_str(&format!("\"{}\":{}", location, hv.to_json()));
    }
    data_json.push_str("}");

    let response = format!("{{\"code\":200,\"msg\":\"\",\"data\":{}}}", data_json);
    ctx.ok(response.as_bytes());
}

// 查询行程数据
#[no_mangle]
pub extern "C" fn query() {
    let ctx = &mut sim_context::get_sim_context();
    let phone = ctx.arg_as_utf8_str("phone");

    if is_blank(&phone) {
        ctx.error("phone cannot be empty");
        return;
    }

    let data = match ctx.get_state(&phone, "") {
        Ok(d) => d,
        Err(e) => {
            ctx.error("get state failed");
            return;
        }
    };

    if data.is_empty() {
        ctx.ok("{\"code\":404,\"msg\":\"not found\",\"data\":null}".as_bytes());
        return;
    }

    let json_str = String::from_utf8(data).unwrap_or("{}".to_string());
    let response = format!("{{\"code\":200,\"msg\":\"\",\"data\":{}}}", json_str);
    ctx.ok(response.as_bytes());
}