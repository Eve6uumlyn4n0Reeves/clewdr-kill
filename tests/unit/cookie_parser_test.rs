// Cookie 解析单元测试示例

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use clewdr::config::ClewdrCookie;

    #[test]
    fn test_parse_valid_cookie() {
        let cookie_str = "sk-ant-sid01-ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-AAAAAA";
        let result = ClewdrCookie::from_str(cookie_str);
        assert!(result.is_ok(), "有效格式应解析成功");
    }

    #[test]
    fn test_parse_invalid_cookie_format() {
        let invalid = "invalid-cookie";
        let result = ClewdrCookie::from_str(invalid);
        assert!(result.is_err(), "无效格式应返回错误");
    }

    #[test]
    fn test_parse_empty_cookie() {
        let result = ClewdrCookie::from_str("");
        assert!(result.is_err(), "空字符串应返回错误");
    }
}
