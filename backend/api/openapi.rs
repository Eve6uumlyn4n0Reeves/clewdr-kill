//! Static OpenAPI document (minimal覆盖核心接口)
//! 便于 swagger/redoc 展示与二次开发

pub const OPENAPI_JSON: &str = r#"{
  "openapi": "3.0.3",
  "info": {
    "title": "ClewdR Kill Edition API",
    "version": "0.11.27",
    "description": "Admin-only API for Claude cookie banning"
  },
  "paths": {
    "/api/auth/login": {
      "post": {
        "summary": "登录获取JWT",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": { "type": "object", "properties": { "password": { "type": "string" } }, "required": ["password"] }
            }
          }
        },
        "responses": {
          "200": {
            "description": "登录成功",
            "content": { "application/json": { "schema": { "$ref": "#/components/schemas/LoginResponse" } } }
          },
          "401": { "description": "认证失败" }
        }
      }
    },
    "/api/cookies": {
      "get": {
        "summary": "获取队列状态",
        "responses": {
          "200": { "description": "队列信息", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/QueueStatusResponse" } } } },
          "401": { "description": "未认证" }
        },
        "security": [{ "bearerAuth": [] }]
      }
    },
    "/api/cookies/batch": {
      "post": {
        "summary": "批量提交Cookie",
        "requestBody": {
          "required": true,
          "content": { "application/json": { "schema": { "$ref": "#/components/schemas/BatchSubmitRequest" } } }
        },
        "responses": {
          "200": { "description": "批量提交结果", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/BatchSubmitResult" } } } },
          "401": { "description": "未认证" }
        },
        "security": [{ "bearerAuth": [] }]
      }
    },
    "/api/stats/system": {
      "get": {
        "summary": "系统统计",
        "responses": {
          "200": { "description": "系统统计", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/SystemStats" } } } }
        },
        "security": [{ "bearerAuth": [] }]
      }
    },
    "/api/config": {
      "get": {
        "summary": "获取配置",
        "responses": {
          "200": { "description": "配置", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ConfigResponse" } } } }
        },
        "security": [{ "bearerAuth": [] }]
      },
      "post": {
        "summary": "更新配置",
        "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ConfigUpdateRequest" } } } },
        "responses": { "200": { "description": "更新后的配置" } },
        "security": [{ "bearerAuth": [] }]
      }
    },
    "/api/prompts/save": {
      "post": {
        "summary": "保存提示词",
        "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/SavePromptRequest" } } } },
        "responses": { "200": { "description": "保存成功" } },
        "security": [{ "bearerAuth": [] }]
      }
    },
    "/metrics": {
      "get": {
        "summary": "Prometheus metrics",
        "responses": { "200": { "description": "text format", "content": { "text/plain": {} } } }
      }
    }
  },
  "components": {
    "schemas": {
      "LoginResponse": {
        "type": "object",
        "properties": {
          "token": { "type": "string" },
          "expires_at": { "type": "string", "format": "date-time" }
        }
      },
      "BatchSubmitRequest": {
        "type": "object",
        "properties": {
          "cookies": { "type": "array", "items": { "type": "string" } },
          "batch_size": { "type": "integer" }
        },
        "required": ["cookies"]
      },
      "BatchSubmitResult": {
        "type": "object",
        "properties": {
          "success": { "type": "integer" },
          "failed": { "type": "integer" },
          "total": { "type": "integer" },
          "errors": { "type": "array", "items": { "type": "string" } }
        }
      },
      "QueueStatusResponse": {
        "type": "object",
        "properties": {
          "pending": { "type": "array", "items": { "type": "object" } },
          "processing": { "type": "array", "items": { "type": "object" } },
          "banned": { "type": "array", "items": { "type": "object" } },
          "total_requests": { "type": "integer" }
        }
      },
      "SystemStats": {
        "type": "object",
        "properties": {
          "total_cookies": { "type": "integer" },
          "pending_cookies": { "type": "integer" },
          "banned_cookies": { "type": "integer" },
          "total_requests": { "type": "integer" },
          "requests_per_minute": { "type": "number" },
          "success_rate": { "type": "number" },
          "average_response_time": { "type": "integer" },
          "workers_active": { "type": "integer" },
          "uptime_seconds": { "type": "integer" }
        }
      },
      "ConfigResponse": {
        "type": "object",
        "properties": {
          "ban_config": { "type": "object" },
          "server_config": { "type": "object" },
          "network_config": { "type": "object" }
        }
      },
      "ConfigUpdateRequest": {
        "type": "object",
        "properties": {
          "ban_config": { "type": "object" },
          "server_config": { "type": "object" },
          "network_config": { "type": "object" }
        }
      },
      "SavePromptRequest": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "content": { "type": "string" }
        },
        "required": ["name", "content"]
      }
    },
    "securitySchemes": {
      "bearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    }
  }
}
"#;

use axum::response::{IntoResponse, Response};
use http::{header, HeaderValue, StatusCode};

/// 返回 OpenAPI JSON
pub async fn api_openapi() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(OPENAPI_JSON.into())
        .unwrap()
}
