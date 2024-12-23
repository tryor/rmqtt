[English](../en_US/http-api.md)  | 简体中文

# HTTP API

RMQTT 提供了 HTTP API 以实现与外部系统的集成，例如查询客户端信息、发布消息等。

RMQTT 的 HTTP API 服务默认监听 6060 端口，可通过 `etc/plugins/rmqtt-http-api.toml` 配置文件修改监听端口。所有 API 调用均以 `api/v1` 开头。

#### 插件：

```bash
rmqtt-http-api
```

#### 插件配置文件：

```bash
plugins/rmqtt-http-api.toml
```

#### 插件配置项：

```bash
##--------------------------------------------------------------------
## rmqtt-http-api
##--------------------------------------------------------------------

# See more keys and their definitions at https://github.com/rmqtt/rmqtt/blob/master/docs/en_US/http-api.md

##Number of worker threads
workers = 1
## Max Row Limit
max_row_limit = 10_000
## HTTP Listener
http_laddr = "0.0.0.0:6060"
## Indicates whether to print HTTP request logs
http_request_log = false
## If set, will check request header Authorization value == Bearer $http_bearer_token, default value is undefined
#http_bearer_token = bearer_token

##Message expiration time, 0 means no expiration
message_expiry_interval = "5m"
```

## 响应码

### HTTP 状态码 (status codes)

RMQTT 接口在调用成功时总是返回 200 OK，响应内容主要以 JSON 格式返回。

可能的状态码如下：

| Status Code | Description                               |
| ---- |-------------------------------------------|
| 200  | 成功，如果需要返回更多数据，将以 JSON 数据格式返回              |
| 400  | 客户端请求无效，例如请求体或参数错误                        |
| 401  | 客户端未通过服务端认证，使用无效的身份验证凭据可能会发生              |
| 404  | 找不到请求的路径或者请求的对象不存在                        |
| 500  | 服务端处理请求时发生内部错误                            |

## API Endpoints

## /api/v1

### GET /api/v1

返回 RMQTT 支持的所有 Endpoints。

**Parameters:** 无

**Success Response Body (JSON):**

| Name             | Type |  Description   |
|------------------| --------- | -------------- |
| []             | Array     | Endpoints 列表 |
| - [0].path   | String    | Endpoint       |
| - [0].name   | String    | Endpoint 名    |
| - [0].method | String    | HTTP Method    |
| - [0].descr  | String    | 描述           |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1"

[{"descr":"Return the basic information of all nodes in the cluster","method":"GET","name":"get_brokers","path":"/brokers/{node}"}, ...]

```

## Broker 基本信息

### GET /api/v1/brokers/{node}

返回集群下所有节点的基本信息。

**Path Parameters:**

| Name | Type | Required | Description                 |
| ---- | --------- | ------------|-----------------------------|
| node | Integer    | False       | 节点ID，如：1 <br/>不指定时返回所有节点的基本信息 |

**Success Response Body (JSON):**

| Name           | Type | Description                                            |
|----------------| --------- |--------------------------------------------------------|
| {}/[]          | Object/Array of Objects | node 参数存在时返回指定节点信息，<br/>不存在时返回所有节点的信息                  |
| .datetime      | String    | 当前时间，格式为 "YYYY-MM-DD HH:mm:ss"                         |
| .node_id       | Integer    | 节点ID                                                   |
| .node_name     | String    | 节点名称                                                   |
| .running       | Bool    | 节点是否正常                                                   |
| .sysdescr      | String    | 软件描述                                                   |
| .uptime        | String    | RMQTT 运行时间，格式为 "D days, H hours, m minutes, s seconds" |
| .version       | String    | RMQTT 版本                                               |
| .rustc_version | String    | RUSTC 版本                                               |


**Examples:**

获取所有节点的基本信息：

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/brokers"

[{"datetime":"2022-07-24 23:01:31","node_id":1,"node_name":"1@127.0.0.1","node_status":"Running","sysdescr":"RMQTT Broker","uptime":"5 days 23 hours, 16 minutes, 3 seconds","version":"rmqtt/0.2.3-20220724094535"}]
```

获取节点 1 的基本信息：

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/brokers/1"

{"datetime":"2022-07-24 23:01:31","node_id":1,"node_name":"1@127.0.0.1","node_status":"Running","sysdescr":"RMQTT Broker","uptime":"5 days 23 hours, 17 minutes, 15 seconds","version":"rmqtt/0.2.3-20220724094535"}
```

## 节点

### GET /api/v1/nodes/{node}

返回节点的状态。

**Path Parameters:**

| Name | Type | Required | Description                 |
| ---- | --------- | ------------|-----------------------------|
| node | Integer    | False       | 节点ID，如：1 <br/>不指定时返回所有节点的信息 |

**Success Response Body (JSON):**

| Name            | Type                    | Description                                     |
|-----------------|-------------------------|-------------------------------------------------|
| {}/[]           | Object/Array of Objects | node 参数存在时返回指定节点信息，<br/>不存在时以 Array 形式返回所有节点的信息 |
| .boottime       | String                  | 操作系统启动时间                                        |
| .connections    | Integer                 | 当前接入此节点的客户端数量                                   |
| .disk_free      | Integer                 | 磁盘可用容量（字节）                                      |
| .disk_total     | Integer                 | 磁盘总容量（字节）                                       |
| .load1          | Float                   | 1 分钟内的 CPU 平均负载                                 |
| .load5          | Float                   | 5 分钟内的 CPU 平均负载                                 |
| .load15         | Float                   | 15 分钟内的 CPU 平均负载                                |
| .memory_free    | Integer                 | 系统可用内存大小（字节）                                    |
| .memory_total   | Integer                 | 系统总内存大小（字节）                                     |
| .memory_used    | Integer                 | 系统已占用的内存大小 （字节）                                 |
| .node_id        | Integer                 | 节点ID                                            |
| .node_name      | String                  | 节点名称                                            |
| .running        | Bool                    | 节点是否正常                                          |
| .uptime         | String                  | RMQTT 运行时间                                      |
| .version        | String                  | RMQTT 版本                                        |
| .rustc_version  | String                  | RUSTC 版本                                        |

**Examples:**

获取所有节点的状态：

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/nodes"

[{"boottime":"2022-06-30 05:20:24 UTC","connections":1,"disk_free":77382381568,"disk_total":88692346880,"load1":0.0224609375,"load15":0.0,"load5":0.0263671875,"memory_free":1457954816,"memory_total":2084057088,"memory_used":626102272,"node_id":1,"node_name":"1@127.0.0.1","node_status":"Running","uptime":"5 days 23 hours, 33 minutes, 0 seconds","version":"rmqtt/0.2.3-20220724094535"}]
```

获取指定节点的状态：

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/nodes/1"

{"boottime":"2022-06-30 05:20:24 UTC","connections":1,"disk_free":77382381568,"disk_total":88692346880,"load1":0.0224609375,"load15":0.0,"load5":0.0263671875,"memory_free":1457954816,"memory_total":2084057088,"memory_used":626102272,"node_id":1,"node_name":"1@127.0.0.1","node_status":"Running","uptime":"5 days 23 hours, 33 minutes, 0 seconds","version":"rmqtt/0.2.3-20220724094535"}
```

## 客户端

### GET /api/v1/clients

<span id = "get-clients" />

返回集群下所有客户端的信息。

**Query String Parameters:**

| Name   | Type | Required | Default | Description |
| ------ | --------- | -------- | ------- |  ---- |
| _limit | Integer   | False | 10000   | 一次最多返回的数据条数，未指定时由 `rmqtt-http-api.toml` 插件的配置项 `max_row_limit` 决定 |

| Name            | Type   | Required | Description         |
| --------------- | ------ | -------- |---------------------|
| clientid        | String | False    | 客户端标识符              |
| username        | String | False    | 客户端用户名              |
| ip_address      | String | False    | 客户端 IP 地址           |
| connected       | Bool   | False    | 客户端当前连接状态           |
| clean_start     | Bool   | False    | 客户端是否使用了全新的会话       |
| session_present | Bool   | False    | 客户端是否连接到已经存在的会话    |
| proto_ver       | Integer| False    | 客户端协议版本, 3,4,5      |
| _like_clientid  | String | False    | 客户端标识符，子串方式模糊查找     |
| _like_username  | String | False    | 客户端用户名，子串方式模糊查找     |
| _gte_created_at | Integer| False    | 客户端会话创建时间，大于等于查找    |
| _lte_created_at | Integer| False    | 客户端会话创建时间，小于等于查找    |
| _gte_connected_at | Integer| False    | 客户端连接创建时间，大于等于查找    |
| _lte_connected_at | Integer| False    | 客户端连接创建时间，小于等于查找    |
| _gte_mqueue_len | Integer| False    | 客户端消息队列当前长度， 大于等于查找 |
| _lte_mqueue_len | Integer| False    | 客户端消息队列当前长度， 大于等于查找 |

**Success Response Body (JSON):**

| Name                    | Type             | Description                                                                |
|-------------------------|------------------|----------------------------------------------------------------------------|
| []                      | Array of Objects | 所有客户端的信息                                                                   |
| [0].node_id             | Integer          | 客户端所连接的节点ID                                                                |
| [0].clientid            | String           | 客户端标识符                                                                     |
| [0].username            | String           | 客户端连接时使用的用户名                                                               | 
| [0].proto_ver           | Integer          | 客户端使用的协议版本                                                                 |
| [0].ip_address          | String           | 客户端的 IP 地址                                                                 |
| [0].port                | Integer          | 客户端的端口                                                                     | 
| [0].connected_at        | String           | 客户端连接时间，格式为 "YYYY-MM-DD HH:mm:ss"                                          |
| [0].disconnected_at     | String           | 客户端离线时间，格式为 "YYYY-MM-DD HH:mm:ss"，<br/>此字段仅在 `connected` 为 `false` 时有效并被返回 |
| [0].disconnected_reason | String           | 客户端离线原因                                                                    |
| [0].connected           | Boolean          | 客户端是否处于连接状态                                                                |
| [0].keepalive           | Integer          | 保持连接时间，单位：秒                                                                |
| [0].clean_start         | Boolean          | 指示客户端是否使用了全新的会话                                                            |
| [0].expiry_interval     | Integer          | 会话过期间隔，单位：秒                                                                |
| [0].created_at          | String           | 会话创建时间，格式为 "YYYY-MM-DD HH:mm:ss"                                           |
| [0].subscriptions_cnt   | Integer          | 此客户端已建立的订阅数量                                                               |
| [0].max_subscriptions   | Integer          | 此客户端允许建立的最大订阅数量                                                            |
| [0].inflight            | Integer          | 飞行队列当前长度                                                                   |
| [0].max_inflight        | Integer          | 飞行队列最大长度                                                                   |
| [0].mqueue_len          | Integer          | 消息队列当前长度                                                                   |
| [0].max_mqueue          | Integer          | 消息队列最大长度                                                                   |
| [0].extra_attrs         | Integer          | 扩展属性数量                                                                     |
| [0].last_will           | Json             | 遗嘱消息, 例如：{ "message": "dGVzdCAvdGVzdC9sd3QgLi4u", "qos": 1, "retain": false, "topic": "/test/lwt" } |


**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/clients?_limit=10"

[{"clean_start":true,"clientid":"be82ee31-7220-4cad-a724-aaad9a065012","connected":true,"connected_at":"2022-07-30 18:14:08","created_at":"2022-07-30 18:14:08","disconnected_at":"","expiry_interval":7200,"inflight":0,"ip_address":"183.193.169.110","keepalive":60,"max_inflight":16,"max_mqueue":1000,"max_subscriptions":0,"mqueue_len":0,"node_id":1,"port":10839,"proto_ver":4,"subscriptions_cnt":0,"username":"undefined"}]
```

### GET /api/v1/clients/{clientid}

返回指定客户端的信息

**Path Parameters:**

| Name   | Type | Required | Description |
| ------ | --------- | -------- |  ---- |
| clientid  | String | True | ClientID |

**Success Response Body (JSON):**

| Name | Type | Description |
|------| --------- | ----------- |
| {}   | Array of Objects | 客户端的信息，详细请参见<br/>[GET /api/v1/clients](#get-clients)|

**Examples:**

查询指定客户端

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/clients/example1"

{"clean_start":true,"clientid":"example1","connected":true,"connected_at":"2022-07-30 23:30:43","created_at":"2022-07-30 23:30:43","disconnected_at":"","expiry_interval":7200,"inflight":0,"ip_address":"183.193.169.110","keepalive":60,"max_inflight":16,"max_mqueue":1000,"max_subscriptions":0,"mqueue_len":0,"node_id":1,"port":11232,"proto_ver":4,"subscriptions_cnt":0,"username":"undefined"}
```

### DELETE /api/v1/clients/{clientid}

踢除指定客户端。注意踢除客户端操作会将连接与会话一并终结。

**Path Parameters:**

| Name   | Type | Required | Description |
| ------ | --------- | -------- |  ---- |
| clientid  | String | True | ClientID |

**Success Response Body (String):**

| Name       | Type             | Description |
|------------|------------------|-----------|
| id         | String          | 连接唯一ID    |

**Examples:**

踢除指定客户端

```bash
$ curl -i -X DELETE "http://localhost:6060/api/v1/clients/example1"

1@10.0.4.6:1883/183.193.169.110:10876/example1/dashboard
```

### GET /api/v1/clients/{clientid}/online

检查客户端是否在线

**Path Parameters:**

| Name   | Type | Required | Description |
| ------ | --------- | -------- |  ---- |
| clientid  | String | True | ClientID |

**Success Response Body (JSON):**

| Name | Type | Description |
|------|------|-------------|
| body | Bool | 是否在线        |

**Examples:**

检查客户端是否在线

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/clients/example1/online"

false
```

## 订阅信息

### GET /api/v1/subscriptions

返回集群下所有订阅信息。

**Query String Parameters:**

| Name   | Type | Required | Default | Description                                                                      |
| ------ | --------- | -------- | ------- |----------------------------------------------------------------------------------|
| _limit | Integer   | False | 10000   | 一次最多返回的数据条数，未指定时由 `rmqtt-http-api.toml` 插件的配置项 `max_row_limit` 决定 |

| Name         | Type    | Description |
| ------------ | ------- | ----------- |
| clientid     | String  | 客户端标识符   |
| topic        | String  | 主题，全等查询 |
| qos          | Enum    | 可取值为：`0`,`1`,`2` |
| share        | String  | 共享订阅的组名称 |
| _match_topic | String  | 主题，匹配查询 |

**Success Response Body (JSON):**

| Name            | Type             | Description |
|-----------------|------------------|-------------|
| []              | Array of Objects | 所有订阅信息      |
| [0].node_id     | Integer          | 节点ID        |
| [0].clientid    | String           | 客户端标识符      |
| [0].client_addr | String           | 客户端IP地址和端口  |
| [0].topic       | String           | 订阅主题        |
| [0].qos         | Integer          | QoS 等级      |
| [0].share       | String           | 共享订阅的组名称    |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/subscriptions?_limit=10"

[{"node_id":1,"clientid":"example1","topic":"foo/#","qos":2,"share":null},{"node_id":1,"clientid":"example1","topic":"foo/+","qos":2,"share":"test"}]
```

### GET /api/v1/subscriptions/{clientid}

返回集群下指定客户端的订阅信息。

**Path Parameters:**

| Name   | Type | Required | Description |
| ------ | --------- | -------- |  ---- |
| clientid  | String | True | ClientID |

**Success Response Body (JSON):**

| Name            | Type             | Description |
|-----------------|------------------|-------------|
| []              | Array of Objects | 所有订阅信息      |
| [0].node_id     | Integer          | 节点ID        |
| [0].clientid    | String           | 客户端标识符      |
| [0].client_addr | String           | 客户端IP地址和端口  |
| [0].topic       | String           | 订阅主题        |
| [0].qos         | Integer          | QoS 等级      |
| [0].share       | String           | 共享订阅的组名称      |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/subscriptions/example1"

[{"node_id":1,"clientid":"example1","topic":"foo/+","qos":2,"share":"test"},{"node_id":1,"clientid":"example1","topic":"foo/#","qos":2,"share":null}]
```

## 路由

### GET /api/v1/routes

返回集群下的所有路由信息。

**Query String Parameters:**

| Name   | Type | Required | Default | Description |
| ------ | --------- | -------- | ------- |  ---- |
| _limit | Integer   | False | 10000   | 一次最多返回的数据条数，未指定时由 `rmqtt-http-api.toml` 插件的配置项 `max_row_limit` 决定 |

**Success Response Body (JSON):**

| Name          | Type | Description |
|---------------| --------- |-------------|
| []            | Array of Objects | 所有路由信息      |
| [0].topic | String    | MQTT 主题     |
| [0].node_id  | Integer    | 节点ID        |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/routes"

[{"node_id":1,"topic":"foo/#"},{"node_id":1,"topic":"foo/+"}]
```

### GET /api/v1/routes/{topic}

返回集群下指定主题的路由信息。

**Path Parameters:**

| Name   | Type | Required | Description |
| ------ | --------- | -------- |  ---- |
| topic  | String   | True | 主题 |

**Success Response Body (JSON):**

| Name      | Type | Description |
|-----------| --------- |-------------|
| []        | Array of Objects | 所有路由信息      |
| [0].topic | String    | MQTT 主题     |
| [0].node_id | Integer    | 节点ID        |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/routes/foo%2f1"

[{"node_id":1,"topic":"foo/#"},{"node_id":1,"topic":"foo/+"}]
```

## 消息发布

### POST /api/v1/mqtt/publish

发布 MQTT 消息。

**Parameters (json):**

| Name     | Type | Required | Default | Description                             |
| -------- | --------- | -------- |--------|-----------------------------------------|
| topic    | String    | Optional |        | 主题，与 `topics` 至少指定其中之一                  |
| topics   | String    | Optional |        | 以 `,` 分割的多个主题，使用此字段能够同时发布消息到多个主题        |
| clientid | String    | Optional | system | 客户端标识符                            |
| payload  | String    | Required |        | 消息正文                                    |
| encoding | String    | Optional | plain  | 消息正文使用的编码方式，目前仅支持 `plain` 与 `base64` 两种 |
| qos      | Integer   | Optional | 0      | QoS 等级                                  |
| retain   | Boolean   | Optional | false  | 是否为保留消息                                 |

**Success Response Body (JSON):**

| Name | Type   | Description |
|------|--------|-------------|
| body | String | ok          |

**Examples:**

```bash
$ curl -i -X POST "http://localhost:6060/api/v1/mqtt/publish" --header 'Content-Type: application/json' -d '{"topic":"foo/1","payload":"Hello World","qos":1,"retain":false,"clientid":"example"}'

ok

$ curl -i -X POST "http://localhost:6060/api/v1/mqtt/publish" --header 'Content-Type: application/json' -d '{"topic":"foo/1","payload":"SGVsbG8gV29ybGQ=","qos":1,"encoding":"base64"}'

ok
```

## 主题订阅

### POST /api/v1/mqtt/subscribe

订阅 MQTT 主题。

**Parameters (json):**

| Name     | Type | Required | Default | Description |
| -------- | --------- | -------- | ------- | ------------ |
| topic    | String    | Optional |         | 主题，与 `topics` 至少指定其中之一 |
| topics   | String    | Optional |         | 以 `,` 分割的多个主题，使用此字段能够同时订阅多个主题 |
| clientid | String    | Required |         | 客户端标识符 |
| qos      | Integer   | Optional | 0       | QoS 等级 |

**Success Response Body (JSON):**

| Name    | Type   | Description               |
|---------|--------|---------------------------|
| {}      | Object |                           |
| {topic} | Bool   | key为主题，值为订阅结果: true/false |

**Examples:**

同时订阅 `foo/a`, `foo/b`, `foo/c` 三个主题

```bash
$ curl -i -X POST "http://localhost:6060/api/v1/mqtt/subscribe" --header 'Content-Type: application/json' -d '{"topics":"foo/a,foo/b,foo/c","qos":1,"clientid":"example1"}'

{"foo/a":true,"foo/c":true,"foo/b":true}
```

### POST /api/v1/mqtt/unsubscribe

取消订阅。

**Parameters (json):**

| Name     | Type | Required | Default | Description  |
| -------- | --------- | -------- | ------- | ------------ |
| topic    | String    | Required |         | 主题         |
| clientid | String    | Required |         | 客户端标识符 |

**Success Response Body (JSON):**

| Name | Type | Description |
|------|------|-------------|
| body | Bool | true/false  |

**Examples:**

取消订阅 `foo/a` 主题

```bash
$ curl -i -X POST "http://localhost:6060/api/v1/mqtt/unsubscribe" --header 'Content-Type: application/json' -d '{"topic":"foo/a","clientid":"example1"}'

true
```

## 插件

### GET /api/v1/plugins

返回集群下的所有插件信息。

**Path Parameters:** 无

**Success Response Body (JSON):**

| Name                  | Type             | Description                      |
|-----------------------|------------------|----------------------------------|
| []                    | Array of Objects | 所有插件信息                           |
| [0].node              | Integer          | 节点ID                             |
| [0].plugins           | Array            | 插件信息，由对象组成的数组，见下文                |
| [0].plugins.name      | String           | 插件名称                             |
| [0].plugins.version   | String           | 插件版本                             |
| [0].plugins.descr     | String           | 插件描述                             |
| [0].plugins.active    | Boolean          | 插件是否启动                           |
| [0].plugins.inited    | Boolean          | 插件是否已经初始化                        |
| [0].plugins.immutable | Boolean          | 插件是否不可变，不可变插件将不能被停止，不能修改配置，不能重启等 |
| [0].plugins.attrs     | Json             | 插件其它附加属性                         |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/plugins"

[{"node":1,"plugins":[{"active":false,"attrs":null,"descr":null,"immutable":true,"inited":false,"name":"rmqtt-cluster-raft","version":null},{"active":false,"attrs":null,"descr":null,"immutable":false,"inited":false,"name":"rmqtt-auth-http","version":null},{"active":true,"attrs":null,"descr":"","immutable":true,"inited":true,"name":"rmqtt-acl","version":"0.1.1"},{"active":true,"attrs":null,"descr":"","immutable":false,"inited":true,"name":"rmqtt-counter","version":"0.1.0"},{"active":true,"attrs":null,"descr":"","immutable":false,"inited":true,"name":"rmqtt-http-api","version":"0.1.1"},{"active":false,"attrs":null,"descr":null,"immutable":false,"inited":false,"name":"rmqtt-web-hook","version":null},{"active":false,"attrs":null,"descr":null,"immutable":true,"inited":false,"name":"rmqtt-cluster-broadcast","version":null}]}]
```

### GET /api/v1/plugins/{node}

返回指定节点下的插件信息。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- |----------|-------------|
| node | Integer    | True     | 节点ID，如：1    |

**Success Response Body (JSON):**

| Name           | Type             | Description                    |
|----------------|------------------|--------------------------------|
| []             | Array of Objects | 插件信息，由对象组成的数组，见下文      |
| [0].name       | String           | 插件名称                           |
| [0].version    | String           | 插件版本                           |
| [0].descr      | String           | 插件描述                           |
| [0].active     | Boolean          | 插件是否启动                         |
| [0].inited     | Boolean          | 插件是否已经初始化                      |
| [0].immutable  | Boolean          | 插件是否不可变，不可变插件将不能被停止，不有修改配置，不能重启等 |
| [0].attrs      | Json             | 插件其它附加属性                       |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/plugins/1"

[{"active":false,"attrs":null,"descr":null,"immutable":true,"inited":false,"name":"rmqtt-cluster-raft","version":null},{"active":false,"attrs":null,"descr":null,"immutable":false,"inited":false,"name":"rmqtt-auth-http","version":null},{"active":true,"attrs":null,"descr":"","immutable":true,"inited":true,"name":"rmqtt-acl","version":"0.1.1"},{"active":true,"attrs":null,"descr":"","immutable":false,"inited":true,"name":"rmqtt-counter","version":"0.1.0"},{"active":true,"attrs":null,"descr":"","immutable":false,"inited":true,"name":"rmqtt-http-api","version":"0.1.1"},{"active":false,"attrs":null,"descr":null,"immutable":false,"inited":false,"name":"rmqtt-web-hook","version":null},{"active":false,"attrs":null,"descr":null,"immutable":true,"inited":false,"name":"rmqtt-cluster-broadcast","version":null}]
```

### GET /api/v1/plugins/{node}/{plugin}

返回指定节点下指定插件名称的插件信息。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |
| plugin | String    | True       | 插件名称        |

**Success Response Body (JSON):**

| Name           | Type            | Description                    |
|----------------|-----------------|--------------------------------|
| {}             | Object | 插件信息      |
| {}.name       | String          | 插件名称                           |
| {}.version    | String          | 插件版本                           |
| {}.descr      | String          | 插件描述                           |
| {}.active     | Boolean         | 插件是否启动                         |
| {}.inited     | Boolean         | 插件是否已经初始化                      |
| {}.immutable  | Boolean         | 插件是否不可变，不可变插件将不能被停止，不有修改配置，不能重启等 |
| {}.attrs      | Json            | 插件其它附加属性                       |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/plugins/1/rmqtt-web-hook"

{"active":false,"attrs":null,"descr":null,"immutable":false,"inited":false,"name":"rmqtt-web-hook","version":null}
```

### GET /api/v1/plugins/{node}/{plugin}/config

返回指定节点下指定插件名称的插件配置信息。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |
| plugin | String    | True       | 插件名称        |

**Success Response Body (JSON):**

| Name           | Type     | Description |
|----------------|----------|-------------|
| {}             | Object   | 插件配置信息      |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/plugins/1/rmqtt-http-api/config"

{"http_laddr":"0.0.0.0:6060","max_row_limit":10000,"workers":1}
```

### PUT /api/v1/plugins/{node}/{plugin}/config/reload

重新载入指定节点下指定插件名称的插件配置信息。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |
| plugin | String    | True       | 插件名称        |

**Success Response Body (String):**

| Name | Type   | Description |
|------|--------|-------------|
| body | String | ok          |

**Examples:**

```bash
$ curl -i -X PUT "http://localhost:6060/api/v1/plugins/1/rmqtt-http-api/config/reload"

ok
```

### PUT /api/v1/plugins/{node}/{plugin}/load

加载指定节点下的指定插件。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |
| plugin | String    | True       | 插件名称        |

**Success Response Body (String):**

| Name | Type   | Description |
|------|--------|-------------|
| body | String | ok          |

**Examples:**

```bash
$ curl -i -X PUT "http://localhost:6060/api/v1/plugins/1/rmqtt-web-hook/load"

ok
```

### PUT /api/v1/plugins/{node}/{plugin}/unload

卸载指定节点下的指定插件。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |
| plugin | String    | True       | 插件名称        |

**Success Response Body (JSON):**

| Name | Type | Description |
|------|------|-------------|
| body | Bool | true/false  |

**Examples:**

```bash
$ curl -i -X PUT "http://localhost:6060/api/v1/plugins/1/rmqtt-web-hook/unload"

true
```

## 状态

### GET /api/v1/stats

<span id = "get-stats" />

返回集群下所有状态数据。

**Path Parameters:** 无

**Success Response Body (JSON):**

| Name          | Type             | Description   |
|---------------|------------------| ------------- |
| []            | Array of Objects | 各节点上的状态数据列表 |
| [0].node  | Json Object      | 节点信息 |
| [0].stats | Json Object      | 状态数据，详见下面的 *stats* |

**node:**

| Name          | Type    | Description |
|---------------|---------|-------------|
| id            | Integer | 节点ID       |
| name          | String  | 节点名称      |
| status        | String | 节点状态       |

**stats:**

| Name                       | Type | Description            |
|----------------------------| --------- | ---------------------- |
| connections.count          | Integer   | 当前连接数量           |
| connections.max            | Integer   | 连接数量的历史最大值     |
| handshakings.count         | Integer   | 当前握手的连接数量     |
| handshakings.max           | Integer   | 当前握手的连接数量的历史最大值   |
| handshakings_active.count  | Integer   | 当前正在执行握手操作的连接数量   |
| handshakings_rate.count    | Integer   | 连接握手速率       |
| handshakings_rate.max      | Integer   | 连接握手速率的历史最大值     |
| sessions.count             | Integer   | 当前会话数量           |
| sessions.max               | Integer   | 会话数量的历史最大值     |
| topics.count               | Integer   | 当前主题数量           |
| topics.max                 | Integer   | 主题数量的历史最大值     |
| subscriptions.count        | Integer   | 当前订阅数量，包含共享订阅 |
| subscriptions.max          | Integer   | 订阅数量的历史最大值     |
| subscriptions_shared.count | Integer   | 当前共享订阅数量         |
| subscriptions_shared.max   | Integer   | 共享订阅数量的历史最大值 |
| routes.count               | Integer   | 当前路由数量           |
| routes.max                 | Integer   | 路由数量的历史最大值     |
| retained.count             | Integer   | 当前保留消息数量         |
| retained.max               | Integer   | 保留消息的历史最大值     |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/stats"

[{"node":{"id":1,"name":"1@127.0.0.1","status":"Running"},"stats":{"connections.count":1,"connections.max":2,"retained.count":2,"retained.max":2,"routes.count":3,"routes.max":4,"sessions.count":1,"sessions.max":2,"subscriptions.count":7,"subscriptions.max":8,"subscriptions_shared.count":1,"subscriptions_shared.max":2,"topics.count":3,"topics.max":4}}]
```

### GET /api/v1/stats/{node}

返回集群下指定节点的状态数据。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |

**Success Response Body (JSON):**

| Name          | Type                 | Description        |
|---------------|----------------------|--------------------|
| {}            | Object               | 各节点上的状态数据列表        |
| {}.node  | Json Object          | 节点信息               |
| {}.stats | Json Object          | 状态数据，详见下面的 *stats* |

**node:**

| Name          | Type    | Description |
|---------------|---------|-------------|
| id            | Integer | 节点ID       |
| name          | String  | 节点名称      |
| status        | String | 节点状态       |

**stats:**

| Name | Type | Description |
|------| --------- | ----------- |
| {}   | Json Object | 状态数据，详细请参见<br/>[GET /api/v1/stats](#get-stats)|

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/stats/1"

{"node":{"id":1,"name":"1@127.0.0.1","status":"Running"},"stats":{"connections.count":1,"connections.max":2,"retained.count":2,"retained.max":2,"routes.count":3,"routes.max":4,"sessions.count":1,"sessions.max":2,"subscriptions.count":7,"subscriptions.max":8,"subscriptions_shared.count":1,"subscriptions_shared.max":2,"topics.count":3,"topics.max":4}}
```

### GET /api/v1/stats/sum

汇总集群下所有节点状态数据。

**Path Parameters:** 无

**Success Response Body (JSON):**

| Name          | Type                 | Description        |
|---------------|----------------------|--------------------|
| {}            | Object               | 各节点上的状态数据列表        |
| {}.nodes  | Json Objects          | 节点信息               |
| {}.stats | Json Object          | 状态数据，详见下面的 *stats* |

**nodes:**

| Name        | Type     | Description    |
|-------------|----------|----------------|
| {id}        | Object   | 节点, key为节点ID  |
| {id}.name   | String   | 节点名称           |
| {id}.status | String   | 节点状态           |

**stats:**

| Name | Type | Description |
|------| --------- | ----------- |
| {}   | Json Object | 状态数据，详细请参见<br/>[GET /api/v1/stats](#get-stats)|

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/stats/sum"

{"nodes":{"1":{"name":"1@127.0.0.1","status":"Running"}},"stats":{"connections.count":1,"connections.max":2,"retained.count":2,"retained.max":2,"routes.count":3,"routes.max":4,"sessions.count":1,"sessions.max":2,"subscriptions.count":7,"subscriptions.max":8,"subscriptions_shared.count":1,"subscriptions_shared.max":2,"topics.count":3,"topics.max":4}}
```

## 统计指标

### GET /api/v1/metrics

<span id = "get-metrics" />

返回集群下所有统计指标数据。

**Path Parameters:** 无

**Success Response Body (JSON):**

| Name          | Type             | Description   |
|---------------|------------------| ------------- |
| []            | Array of Objects | 各节点上的统计指标列表 |
| [0].node  | Json Object      | 节点信息 |
| [0].metrics | Json Object      | 监控指标数据，详见下面的 *metrics* |

**node:**

| Name          | Type    | Description |
|---------------|---------|-------------|
| id            | Integer | 节点ID       |
| name          | String  | 节点名称      |

**metrics:**

| Name                            | Type | Description                      |
|---------------------------------| --------- |----------------------------------|
| client.auth.anonymous           | Integer   | 匿名登录的客户端数量                       |
| client.auth.anonymous.error     | Integer   | 匿名登录失败的客户端数量                     |
| client.authenticate             | Integer   | 客户端认证次数                          |
| client.connack                  | Integer   | 发送 CONNACK 报文的次数                 |
| client.connack.auth.error       | Integer   | 发送连接认证失败的 CONNACK 报文的次数          |
| client.connack.error            | Integer   | 发送连接失败的 CONNACK 报文的次数            |
| client.connect                  | Integer   | 客户端连接次数                          |
| client.connected                | Integer   | 客户端成功连接次数                        |
| client.disconnected             | Integer   | 客户端断开连接次数                        |
| client.handshaking.timeout      | Integer   | 连接握手超时次数                         |
| client.publish.auth.error       | Integer   | 发布，ACL 规则检查失败次数                  |
| client.publish.check.acl        | Integer   | 发布，ACL 规则检查次数                    |
| client.publish.error            | Integer   | 发布，失败次数                          |
| client.subscribe.auth.error     | Integer   | 订阅，ACL 规则检查失败次数                  |
| client.subscribe.error          | Integer   | 订阅，失败次数                          |
| client.subscribe.check.acl      | Integer   | 订阅，ACL 规则检查次数                    |
| client.subscribe                | Integer   | 客户端订阅次数                          |
| client.unsubscribe              | Integer   | 客户端取消订阅次数                        |
| messages.publish                | Integer   | 接收到PUBLISH消息数量                   |
| messages.publish.admin          | Integer   | 接收到PUBLISH消息数量, 通过HTTP-API发布的消息  |
| messages.publish.custom         | Integer   | 接收到PUBLISH消息数量, 通过MQTT客户端发布的消息   |
| messages.publish.lastwill       | Integer   | 接收到PUBLISH消息数量, 遗嘱消息             |
| messages.publish.retain         | Integer   | 接收到PUBLISH消息数量, 转发的保留消息          |
| messages.publish.system         | Integer   | 接收到PUBLISH消息数量, 系统主题消息($SYS/#)   |
| messages.delivered              | Integer   | 向订阅端转发的消息数              |
| messages.delivered.admin        | Integer   | 向订阅端转发的消息数, 通过HTTP-API发布的消息 |
| messages.delivered.custom       | Integer   | 向订阅端转发的消息数, 通过MQTT客户端发布的消息  |
| messages.delivered.lastwill     | Integer   | 向订阅端转发的消息数, 遗嘱消息            |
| messages.delivered.retain       | Integer   | 向订阅端转发的消息数, 转发的保留消息         |
| messages.delivered.system       | Integer   | 向订阅端转发的消息数, 系统主题消息($SYS/#)  |
| messages.acked                  | Integer   | 接收的 PUBACK 和 PUBREC 报文数量                 |
| messages.acked.admin            | Integer   | 接收的 PUBACK 和 PUBREC 报文数量, 通过HTTP-API发布的消息 |
| messages.acked.custom           | Integer   | 接收的 PUBACK 和 PUBREC 报文数量, 通过MQTT客户端发布的消息  |
| messages.acked.lastwill         | Integer   | 接收的 PUBACK 和 PUBREC 报文数量, 遗嘱消息            |
| messages.acked.retain           | Integer   | 接收的 PUBACK 和 PUBREC 报文数量, 转发的保留消息         |
| messages.acked.system           | Integer   | 接收的 PUBACK 和 PUBREC 报文数量, 系统主题消息($SYS/#)  |
| messages.nonsubscribed          | Integer   | 未找到订阅关系的PUBLISH消息数量          |
| messages.nonsubscribed.admin    | Integer   | 未找到订阅关系的PUBLISH消息数量, 通过HTTP-API发布的消息 |
| messages.nonsubscribed.custom   | Integer   | 未找到订阅关系的PUBLISH消息数量, 通过MQTT客户端发布的消息  |
| messages.nonsubscribed.lastwill | Integer   | 未找到订阅关系的PUBLISH消息数量, 遗嘱消息            |
| messages.nonsubscribed.system   | Integer   | 未找到订阅关系的PUBLISH消息数量, 系统主题消息($SYS/#)  |
| messages.dropped                | Integer   | 丢弃的消息总数                                               |
| session.created                 | Integer   | 创建的会话数量                                               |
| session.resumed                 | Integer   | 由于 `Clean Session` 或 `Clean Start` 为 `false` 而恢复的会话数量 |
| session.subscribed              | Integer   | 客户端成功订阅次数                                             |
| session.unsubscribed            | Integer   | 客户端成功取消订阅次数                                           |
| session.terminated              | Integer   | 终结的会话数量                                               |

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/metrics"

[{"metrics":{"client.auth.anonymous":38,"client.authenticate":47,"client.connack":47,"client.connect":47,"client.connected":47,"client.disconnected":46,"client.publish.check.acl":50,"client.subscribe":37,"client.subscribe.check.acl":15,"client.unsubscribe":8,"messages.acked":35,"messages.delivered":78,"messages.dropped":0,"messages.publish":78,"session.created":45,"session.resumed":2,"session.subscribed":15,"session.terminated":42,"session.unsubscribed":8},"node":{"id":1,"name":"1@127.0.0.1"}}]
```

### GET /api/v1/metrics/{node}

返回集群下指定节点的统计指标数据。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | 节点ID，如：1    |

**Success Response Body (JSON):**

| Name          | Type                 | Description            |
|---------------|----------------------|------------------------|
| {}            | Object               | 统计指标信息                   |
| {}.node  | Json Object          | 节点信息                   |
| {}.metrics | Json Object          | 监控指标数据，详见下面的 *metrics* |

**node:**

| Name          | Type    | Description |
|---------------|---------|-------------|
| id            | Integer | 节点ID       |
| name          | String  | 节点名称      |

**metrics:**

| Name | Type | Description |
|------| --------- | ----------- |
| {}   | Json Object | 统计指标数据，详细请参见<br/>[GET /api/v1/metrics](#get-metrics)|

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/metrics/1"

{"metrics":{"client.auth.anonymous":38,"client.authenticate":47,"client.connack":47,"client.connect":47,"client.connected":47,"client.disconnected":46,"client.publish.check.acl":50,"client.subscribe":37,"client.subscribe.check.acl":15,"client.unsubscribe":8,"messages.acked":35,"messages.delivered":78,"messages.dropped":0,"messages.publish":78,"session.created":45,"session.resumed":2,"session.subscribed":15,"session.terminated":42,"session.unsubscribed":8},"node":{"id":1,"name":"1@127.0.0.1"}}
```

### GET /api/v1/metrics/sum

汇总集群下所有节点的统计指标数据。

**Path Parameters:** 无

**Success Response Body (JSON):**

| Name | Type | Description |
|------| --------- | ----------- |
| {}   | Json Object | 统计指标数据，详细请参见<br/>[GET /api/v1/metrics](#get-metrics)|

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/metrics/sum"

{"client.auth.anonymous":38,"client.authenticate":47,"client.connack":47,"client.connect":47,"client.connected":47,"client.disconnected":46,"client.publish.check.acl":50,"client.subscribe":37,"client.subscribe.check.acl":15,"client.unsubscribe":8,"messages.acked":35,"messages.delivered":78,"messages.dropped":0,"messages.publish":78,"session.created":45,"session.resumed":2,"session.subscribed":15,"session.terminated":42,"session.unsubscribed":8}
```

### GET /api/v1/metrics/prometheus

<span id = "get-prometheus" />

以 *prometheus* 格式返回集群中所有节点的状态数据和统计指标数据。

**Path Parameters:** 无

**Success Response Body (TEXT):**

**Examples:**

```bash
$ curl -i -X GET "http://localhost:6060/api/v1/metrics/prometheus"

# HELP rmqtt_metrics All metrics data
# TYPE rmqtt_metrics gauge
rmqtt_metrics{item="client.auth.anonymous",node="1"} 0
rmqtt_metrics{item="client.auth.anonymous",node="2"} 2
rmqtt_metrics{item="client.auth.anonymous",node="3"} 1
rmqtt_metrics{item="client.auth.anonymous",node="all"} 3
rmqtt_metrics{item="client.auth.anonymous.error",node="1"} 0
rmqtt_metrics{item="client.auth.anonymous.error",node="2"} 0
rmqtt_metrics{item="client.auth.anonymous.error",node="3"} 0
rmqtt_metrics{item="client.auth.anonymous.error",node="all"} 0
rmqtt_metrics{item="client.authenticate",node="1"} 1
rmqtt_metrics{item="client.authenticate",node="2"} 2
rmqtt_metrics{item="client.authenticate",node="3"} 1
rmqtt_metrics{item="client.authenticate",node="all"} 4
rmqtt_metrics{item="client.connack",node="1"} 1
rmqtt_metrics{item="client.connack",node="2"} 2
rmqtt_metrics{item="client.connack",node="3"} 1
rmqtt_metrics{item="client.connack",node="all"} 4
rmqtt_metrics{item="client.connack.auth.error",node="1"} 0
rmqtt_metrics{item="client.connack.auth.error",node="2"} 0
rmqtt_metrics{item="client.connack.auth.error",node="3"} 0
rmqtt_metrics{item="client.connack.auth.error",node="all"} 0
rmqtt_metrics{item="client.connack.error",node="1"} 0
rmqtt_metrics{item="client.connack.error",node="2"} 0
rmqtt_metrics{item="client.connack.error",node="3"} 0
rmqtt_metrics{item="client.connack.error",node="all"} 0
rmqtt_metrics{item="client.connect",node="1"} 1
rmqtt_metrics{item="client.connect",node="2"} 2
rmqtt_metrics{item="client.connect",node="3"} 1
rmqtt_metrics{item="client.connect",node="all"} 4
rmqtt_metrics{item="client.connected",node="1"} 1
rmqtt_metrics{item="client.connected",node="2"} 2
rmqtt_metrics{item="client.connected",node="3"} 1
rmqtt_metrics{item="client.connected",node="all"} 4
rmqtt_metrics{item="client.disconnected",node="1"} 0
rmqtt_metrics{item="client.disconnected",node="2"} 0
rmqtt_metrics{item="client.disconnected",node="3"} 0
rmqtt_metrics{item="client.disconnected",node="all"} 0
rmqtt_metrics{item="client.handshaking.timeout",node="1"} 0
rmqtt_metrics{item="client.handshaking.timeout",node="2"} 0
rmqtt_metrics{item="client.handshaking.timeout",node="3"} 0
rmqtt_metrics{item="client.handshaking.timeout",node="all"} 0
rmqtt_metrics{item="client.publish.auth.error",node="1"} 0
rmqtt_metrics{item="client.publish.auth.error",node="2"} 0
rmqtt_metrics{item="client.publish.auth.error",node="3"} 0
rmqtt_metrics{item="client.publish.auth.error",node="all"} 0
rmqtt_metrics{item="client.publish.check.acl",node="1"} 0
rmqtt_metrics{item="client.publish.check.acl",node="2"} 0
rmqtt_metrics{item="client.publish.check.acl",node="3"} 0
rmqtt_metrics{item="client.publish.check.acl",node="all"} 0
rmqtt_metrics{item="client.publish.error",node="1"} 0
rmqtt_metrics{item="client.publish.error",node="2"} 0
rmqtt_metrics{item="client.publish.error",node="3"} 0
rmqtt_metrics{item="client.publish.error",node="all"} 0
rmqtt_metrics{item="client.subscribe",node="1"} 0
rmqtt_metrics{item="client.subscribe",node="2"} 0
rmqtt_metrics{item="client.subscribe",node="3"} 0
rmqtt_metrics{item="client.subscribe",node="all"} 0
rmqtt_metrics{item="client.subscribe.auth.error",node="1"} 0
rmqtt_metrics{item="client.subscribe.auth.error",node="2"} 0
rmqtt_metrics{item="client.subscribe.auth.error",node="3"} 0
rmqtt_metrics{item="client.subscribe.auth.error",node="all"} 0
rmqtt_metrics{item="client.subscribe.check.acl",node="1"} 0
rmqtt_metrics{item="client.subscribe.check.acl",node="2"} 0
rmqtt_metrics{item="client.subscribe.check.acl",node="3"} 0
rmqtt_metrics{item="client.subscribe.check.acl",node="all"} 0
rmqtt_metrics{item="client.subscribe.error",node="1"} 0
rmqtt_metrics{item="client.subscribe.error",node="2"} 0
rmqtt_metrics{item="client.subscribe.error",node="3"} 0
rmqtt_metrics{item="client.subscribe.error",node="all"} 0
rmqtt_metrics{item="client.unsubscribe",node="1"} 0
rmqtt_metrics{item="client.unsubscribe",node="2"} 0
rmqtt_metrics{item="client.unsubscribe",node="3"} 0
rmqtt_metrics{item="client.unsubscribe",node="all"} 0
rmqtt_metrics{item="messages.acked",node="1"} 0
rmqtt_metrics{item="messages.acked",node="2"} 0
rmqtt_metrics{item="messages.acked",node="3"} 0
rmqtt_metrics{item="messages.acked",node="all"} 0
rmqtt_metrics{item="messages.acked.admin",node="1"} 0
rmqtt_metrics{item="messages.acked.admin",node="2"} 0
rmqtt_metrics{item="messages.acked.admin",node="3"} 0
rmqtt_metrics{item="messages.acked.admin",node="all"} 0
rmqtt_metrics{item="messages.acked.bridge",node="1"} 0
rmqtt_metrics{item="messages.acked.bridge",node="2"} 0
rmqtt_metrics{item="messages.acked.bridge",node="3"} 0
rmqtt_metrics{item="messages.acked.bridge",node="all"} 0
rmqtt_metrics{item="messages.acked.custom",node="1"} 0
rmqtt_metrics{item="messages.acked.custom",node="2"} 0
rmqtt_metrics{item="messages.acked.custom",node="3"} 0
rmqtt_metrics{item="messages.acked.custom",node="all"} 0
rmqtt_metrics{item="messages.acked.lastwill",node="1"} 0
rmqtt_metrics{item="messages.acked.lastwill",node="2"} 0
rmqtt_metrics{item="messages.acked.lastwill",node="3"} 0
rmqtt_metrics{item="messages.acked.lastwill",node="all"} 0
rmqtt_metrics{item="messages.acked.retain",node="1"} 0
rmqtt_metrics{item="messages.acked.retain",node="2"} 0
rmqtt_metrics{item="messages.acked.retain",node="3"} 0
rmqtt_metrics{item="messages.acked.retain",node="all"} 0
rmqtt_metrics{item="messages.acked.system",node="1"} 0
rmqtt_metrics{item="messages.acked.system",node="2"} 0
rmqtt_metrics{item="messages.acked.system",node="3"} 0
rmqtt_metrics{item="messages.acked.system",node="all"} 0
rmqtt_metrics{item="messages.delivered",node="1"} 0
rmqtt_metrics{item="messages.delivered",node="2"} 0
rmqtt_metrics{item="messages.delivered",node="3"} 0
rmqtt_metrics{item="messages.delivered",node="all"} 0
rmqtt_metrics{item="messages.delivered.admin",node="1"} 0
rmqtt_metrics{item="messages.delivered.admin",node="2"} 0
rmqtt_metrics{item="messages.delivered.admin",node="3"} 0
rmqtt_metrics{item="messages.delivered.admin",node="all"} 0
rmqtt_metrics{item="messages.delivered.bridge",node="1"} 0
rmqtt_metrics{item="messages.delivered.bridge",node="2"} 0
rmqtt_metrics{item="messages.delivered.bridge",node="3"} 0
rmqtt_metrics{item="messages.delivered.bridge",node="all"} 0
rmqtt_metrics{item="messages.delivered.custom",node="1"} 0
rmqtt_metrics{item="messages.delivered.custom",node="2"} 0
rmqtt_metrics{item="messages.delivered.custom",node="3"} 0
rmqtt_metrics{item="messages.delivered.custom",node="all"} 0
rmqtt_metrics{item="messages.delivered.lastwill",node="1"} 0
rmqtt_metrics{item="messages.delivered.lastwill",node="2"} 0
rmqtt_metrics{item="messages.delivered.lastwill",node="3"} 0
rmqtt_metrics{item="messages.delivered.lastwill",node="all"} 0
rmqtt_metrics{item="messages.delivered.retain",node="1"} 0
rmqtt_metrics{item="messages.delivered.retain",node="2"} 0
rmqtt_metrics{item="messages.delivered.retain",node="3"} 0
rmqtt_metrics{item="messages.delivered.retain",node="all"} 0
rmqtt_metrics{item="messages.delivered.system",node="1"} 0
rmqtt_metrics{item="messages.delivered.system",node="2"} 0
rmqtt_metrics{item="messages.delivered.system",node="3"} 0
rmqtt_metrics{item="messages.delivered.system",node="all"} 0
rmqtt_metrics{item="messages.dropped",node="1"} 0
rmqtt_metrics{item="messages.dropped",node="2"} 0
rmqtt_metrics{item="messages.dropped",node="3"} 0
rmqtt_metrics{item="messages.dropped",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed.admin",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed.admin",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed.admin",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed.admin",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed.bridge",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed.bridge",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed.bridge",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed.bridge",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed.custom",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed.custom",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed.custom",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed.custom",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed.lastwill",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed.lastwill",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed.lastwill",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed.lastwill",node="all"} 0
rmqtt_metrics{item="messages.nonsubscribed.system",node="1"} 0
rmqtt_metrics{item="messages.nonsubscribed.system",node="2"} 0
rmqtt_metrics{item="messages.nonsubscribed.system",node="3"} 0
rmqtt_metrics{item="messages.nonsubscribed.system",node="all"} 0
rmqtt_metrics{item="messages.publish",node="1"} 0
rmqtt_metrics{item="messages.publish",node="2"} 0
rmqtt_metrics{item="messages.publish",node="3"} 0
rmqtt_metrics{item="messages.publish",node="all"} 0
rmqtt_metrics{item="messages.publish.admin",node="1"} 0
rmqtt_metrics{item="messages.publish.admin",node="2"} 0
rmqtt_metrics{item="messages.publish.admin",node="3"} 0
rmqtt_metrics{item="messages.publish.admin",node="all"} 0
rmqtt_metrics{item="messages.publish.bridge",node="1"} 0
rmqtt_metrics{item="messages.publish.bridge",node="2"} 0
rmqtt_metrics{item="messages.publish.bridge",node="3"} 0
rmqtt_metrics{item="messages.publish.bridge",node="all"} 0
rmqtt_metrics{item="messages.publish.custom",node="1"} 0
rmqtt_metrics{item="messages.publish.custom",node="2"} 0
rmqtt_metrics{item="messages.publish.custom",node="3"} 0
rmqtt_metrics{item="messages.publish.custom",node="all"} 0
rmqtt_metrics{item="messages.publish.lastwill",node="1"} 0
rmqtt_metrics{item="messages.publish.lastwill",node="2"} 0
rmqtt_metrics{item="messages.publish.lastwill",node="3"} 0
rmqtt_metrics{item="messages.publish.lastwill",node="all"} 0
rmqtt_metrics{item="messages.publish.system",node="1"} 0
rmqtt_metrics{item="messages.publish.system",node="2"} 0
rmqtt_metrics{item="messages.publish.system",node="3"} 0
rmqtt_metrics{item="messages.publish.system",node="all"} 0
rmqtt_metrics{item="session.created",node="1"} 1
rmqtt_metrics{item="session.created",node="2"} 2
rmqtt_metrics{item="session.created",node="3"} 1
rmqtt_metrics{item="session.created",node="all"} 4
rmqtt_metrics{item="session.resumed",node="1"} 0
rmqtt_metrics{item="session.resumed",node="2"} 0
rmqtt_metrics{item="session.resumed",node="3"} 0
rmqtt_metrics{item="session.resumed",node="all"} 0
rmqtt_metrics{item="session.subscribed",node="1"} 0
rmqtt_metrics{item="session.subscribed",node="2"} 0
rmqtt_metrics{item="session.subscribed",node="3"} 0
rmqtt_metrics{item="session.subscribed",node="all"} 0
rmqtt_metrics{item="session.terminated",node="1"} 0
rmqtt_metrics{item="session.terminated",node="2"} 0
rmqtt_metrics{item="session.terminated",node="3"} 0
rmqtt_metrics{item="session.terminated",node="all"} 0
rmqtt_metrics{item="session.unsubscribed",node="1"} 0
rmqtt_metrics{item="session.unsubscribed",node="2"} 0
rmqtt_metrics{item="session.unsubscribed",node="3"} 0
rmqtt_metrics{item="session.unsubscribed",node="all"} 0
# HELP rmqtt_nodes All nodes status
# TYPE rmqtt_nodes gauge
rmqtt_nodes{item="disk_free",node="1"} 46307106816
rmqtt_nodes{item="disk_free",node="2"} 46307106816
rmqtt_nodes{item="disk_free",node="3"} 46307106816
rmqtt_nodes{item="disk_free",node="all"} 138921320448
rmqtt_nodes{item="disk_total",node="1"} 1000896192512
rmqtt_nodes{item="disk_total",node="2"} 1000896192512
rmqtt_nodes{item="disk_total",node="3"} 1000896192512
rmqtt_nodes{item="disk_total",node="all"} 3002688577536
rmqtt_nodes{item="load1",node="1"} 0
rmqtt_nodes{item="load1",node="2"} 0
rmqtt_nodes{item="load1",node="3"} 0
rmqtt_nodes{item="load1",node="all"} 0
rmqtt_nodes{item="load15",node="1"} 0
rmqtt_nodes{item="load15",node="2"} 0
rmqtt_nodes{item="load15",node="3"} 0
rmqtt_nodes{item="load15",node="all"} 0
rmqtt_nodes{item="load5",node="1"} 0
rmqtt_nodes{item="load5",node="2"} 0
rmqtt_nodes{item="load5",node="3"} 0
rmqtt_nodes{item="load5",node="all"} 0
rmqtt_nodes{item="memory_free",node="1"} 19571781632
rmqtt_nodes{item="memory_free",node="2"} 19571781632
rmqtt_nodes{item="memory_free",node="3"} 19571781632
rmqtt_nodes{item="memory_free",node="all"} 58715344896
rmqtt_nodes{item="memory_total",node="1"} 34070585344
rmqtt_nodes{item="memory_total",node="2"} 34070585344
rmqtt_nodes{item="memory_total",node="3"} 34070585344
rmqtt_nodes{item="memory_total",node="all"} 102211756032
rmqtt_nodes{item="memory_used",node="1"} 14498803712
rmqtt_nodes{item="memory_used",node="2"} 14498803712
rmqtt_nodes{item="memory_used",node="3"} 14498803712
rmqtt_nodes{item="memory_used",node="all"} 43496411136
rmqtt_nodes{item="running",node="1"} 1
rmqtt_nodes{item="running",node="2"} 1
rmqtt_nodes{item="running",node="3"} 1
rmqtt_nodes{item="running",node="all"} 3
# HELP rmqtt_stats All status data
# TYPE rmqtt_stats gauge
rmqtt_stats{item="connections.count",node="1"} 1
rmqtt_stats{item="connections.count",node="2"} 2
rmqtt_stats{item="connections.count",node="3"} 1
rmqtt_stats{item="connections.count",node="all"} 4
rmqtt_stats{item="connections.max",node="1"} 1
rmqtt_stats{item="connections.max",node="2"} 2
rmqtt_stats{item="connections.max",node="3"} 1
rmqtt_stats{item="connections.max",node="all"} 4
rmqtt_stats{item="delayed_publishs.count",node="1"} 0
rmqtt_stats{item="delayed_publishs.count",node="2"} 0
rmqtt_stats{item="delayed_publishs.count",node="3"} 0
rmqtt_stats{item="delayed_publishs.count",node="all"} 0
rmqtt_stats{item="delayed_publishs.max",node="1"} 0
rmqtt_stats{item="delayed_publishs.max",node="2"} 0
rmqtt_stats{item="delayed_publishs.max",node="3"} 0
rmqtt_stats{item="delayed_publishs.max",node="all"} 0
rmqtt_stats{item="forwards.count",node="1"} 0
rmqtt_stats{item="forwards.count",node="2"} 0
rmqtt_stats{item="forwards.count",node="3"} 0
rmqtt_stats{item="forwards.count",node="all"} 0
rmqtt_stats{item="forwards.max",node="1"} 0
rmqtt_stats{item="forwards.max",node="2"} 0
rmqtt_stats{item="forwards.max",node="3"} 0
rmqtt_stats{item="forwards.max",node="all"} 0
rmqtt_stats{item="handshakings.count",node="1"} 0
rmqtt_stats{item="handshakings.count",node="2"} 0
rmqtt_stats{item="handshakings.count",node="3"} 0
rmqtt_stats{item="handshakings.count",node="all"} 0
rmqtt_stats{item="handshakings.max",node="1"} 0
rmqtt_stats{item="handshakings.max",node="2"} 0
rmqtt_stats{item="handshakings.max",node="3"} 0
rmqtt_stats{item="handshakings.max",node="all"} 0
rmqtt_stats{item="handshakings_active.count",node="1"} 0
rmqtt_stats{item="handshakings_active.count",node="2"} 0
rmqtt_stats{item="handshakings_active.count",node="3"} 0
rmqtt_stats{item="handshakings_active.count",node="all"} 0
rmqtt_stats{item="handshakings_rate.count",node="1"} 0
rmqtt_stats{item="handshakings_rate.count",node="2"} 0
rmqtt_stats{item="handshakings_rate.count",node="3"} 0
rmqtt_stats{item="handshakings_rate.count",node="all"} 0
rmqtt_stats{item="handshakings_rate.max",node="1"} 0
rmqtt_stats{item="handshakings_rate.max",node="2"} 0
rmqtt_stats{item="handshakings_rate.max",node="3"} 0
rmqtt_stats{item="handshakings_rate.max",node="all"} 0
rmqtt_stats{item="in_inflights.count",node="1"} 0
rmqtt_stats{item="in_inflights.count",node="2"} 0
rmqtt_stats{item="in_inflights.count",node="3"} 0
rmqtt_stats{item="in_inflights.count",node="all"} 0
rmqtt_stats{item="in_inflights.max",node="1"} 0
rmqtt_stats{item="in_inflights.max",node="2"} 0
rmqtt_stats{item="in_inflights.max",node="3"} 0
rmqtt_stats{item="in_inflights.max",node="all"} 0
rmqtt_stats{item="message_queues.count",node="1"} 0
rmqtt_stats{item="message_queues.count",node="2"} 0
rmqtt_stats{item="message_queues.count",node="3"} 0
rmqtt_stats{item="message_queues.count",node="all"} 0
rmqtt_stats{item="message_queues.max",node="1"} 0
rmqtt_stats{item="message_queues.max",node="2"} 0
rmqtt_stats{item="message_queues.max",node="3"} 0
rmqtt_stats{item="message_queues.max",node="all"} 0
rmqtt_stats{item="message_storages.count",node="1"} -1
rmqtt_stats{item="message_storages.count",node="2"} -1
rmqtt_stats{item="message_storages.count",node="3"} -1
rmqtt_stats{item="message_storages.count",node="all"} -3
rmqtt_stats{item="message_storages.max",node="1"} 0
rmqtt_stats{item="message_storages.max",node="2"} 0
rmqtt_stats{item="message_storages.max",node="3"} 0
rmqtt_stats{item="message_storages.max",node="all"} 0
rmqtt_stats{item="out_inflights.count",node="1"} 0
rmqtt_stats{item="out_inflights.count",node="2"} 0
rmqtt_stats{item="out_inflights.count",node="3"} 0
rmqtt_stats{item="out_inflights.count",node="all"} 0
rmqtt_stats{item="out_inflights.max",node="1"} 0
rmqtt_stats{item="out_inflights.max",node="2"} 0
rmqtt_stats{item="out_inflights.max",node="3"} 0
rmqtt_stats{item="out_inflights.max",node="all"} 0
rmqtt_stats{item="retaineds.count",node="1"} 0
rmqtt_stats{item="retaineds.count",node="2"} 0
rmqtt_stats{item="retaineds.count",node="3"} 0
rmqtt_stats{item="retaineds.count",node="all"} 0
rmqtt_stats{item="retaineds.max",node="1"} 0
rmqtt_stats{item="retaineds.max",node="2"} 0
rmqtt_stats{item="retaineds.max",node="3"} 0
rmqtt_stats{item="retaineds.max",node="all"} 0
rmqtt_stats{item="routes.count",node="1"} 0
rmqtt_stats{item="routes.count",node="2"} 0
rmqtt_stats{item="routes.count",node="3"} 0
rmqtt_stats{item="routes.count",node="all"} 0
rmqtt_stats{item="routes.max",node="1"} 0
rmqtt_stats{item="routes.max",node="2"} 0
rmqtt_stats{item="routes.max",node="3"} 0
rmqtt_stats{item="routes.max",node="all"} 0
rmqtt_stats{item="sessions.count",node="1"} 1
rmqtt_stats{item="sessions.count",node="2"} 2
rmqtt_stats{item="sessions.count",node="3"} 1
rmqtt_stats{item="sessions.count",node="all"} 4
rmqtt_stats{item="sessions.max",node="1"} 1
rmqtt_stats{item="sessions.max",node="2"} 2
rmqtt_stats{item="sessions.max",node="3"} 1
rmqtt_stats{item="sessions.max",node="all"} 4
rmqtt_stats{item="subscriptions.count",node="1"} 0
rmqtt_stats{item="subscriptions.count",node="2"} 0
rmqtt_stats{item="subscriptions.count",node="3"} 0
rmqtt_stats{item="subscriptions.count",node="all"} 0
rmqtt_stats{item="subscriptions.max",node="1"} 0
rmqtt_stats{item="subscriptions.max",node="2"} 0
rmqtt_stats{item="subscriptions.max",node="3"} 0
rmqtt_stats{item="subscriptions.max",node="all"} 0
rmqtt_stats{item="subscriptions_shared.count",node="1"} 0
rmqtt_stats{item="subscriptions_shared.count",node="2"} 0
rmqtt_stats{item="subscriptions_shared.count",node="3"} 0
rmqtt_stats{item="subscriptions_shared.count",node="all"} 0
rmqtt_stats{item="subscriptions_shared.max",node="1"} 0
rmqtt_stats{item="subscriptions_shared.max",node="2"} 0
rmqtt_stats{item="subscriptions_shared.max",node="3"} 0
rmqtt_stats{item="subscriptions_shared.max",node="all"} 0
rmqtt_stats{item="topics.count",node="1"} 0
rmqtt_stats{item="topics.count",node="2"} 0
rmqtt_stats{item="topics.count",node="3"} 0
rmqtt_stats{item="topics.count",node="all"} 0
rmqtt_stats{item="topics.max",node="1"} 0
rmqtt_stats{item="topics.max",node="2"} 0
rmqtt_stats{item="topics.max",node="3"} 0
rmqtt_stats{item="topics.max",node="all"} 0
```

![示例图](../imgs/prometheus_demo1.jpg)

### GET /api/v1/metrics/prometheus/{node}

以 *prometheus* 格式返回集群中指定节点的状态数据和统计指标数据。

**Path Parameters:**

| Name | Type | Required | Description |
| ---- | --------- | ------------|-------------|
| node | Integer    | True       | Node ID, Such as: 1    |

**Success Response Body (TEXT):**

see [GET /api/v1/metrics/prometheus](#get-prometheus) 


### GET /api/v1/metrics/prometheus/sum

以 *prometheus* 格式返回集群中所有节点的状态数据和统计指标数据总和。

**Path Parameters:** 无

**Success Response Body (TEXT):**

see [GET /api/v1/metrics/prometheus](#get-prometheus) 

