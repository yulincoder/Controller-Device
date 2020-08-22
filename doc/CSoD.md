# CSoD(C/S of Device)协议定义端-云交互过程

| Version | Date | Editor | Note |
| :-----| ----: | :----: |:----: |
| v0.0.5 | 2020-05-28 | zhangte | 定义CSoD协议初版|

### TODO List:
- [ ] 状态错误码定义
- [ ] 心跳period实测定义
- [x] 层级定义


## 1. 组成
**IP:** 39.105.63.97 **Port:** 8900
#### 1. 传输层
传输层基于原生Socket/TCP, 承载安全层数据。传输层数据以`\n`作为帧分隔符，即传输每一帧都需要以 `\n`(0xa)作为结束符，服务端以该结束符界定接收结束。
在设备上电后，首先需要与服务端建立可靠传输层连接，该连接需要一直处于保持状态。当客户端检测到传输层中断后，需要重新建立连接。
传输层定义以下建立连接过程与保持连接
###### 1. 连接建立与保持
1. 客户端向服务器端发起建立TCP连接：
2. 客户端向服务端发送心跳Ping包JSON数据，表明身份并告知服务端设备已上线
```JSON
Device -> Server
{
    "v":"0",
    "type": "ping",    
    "sn": "${sn}",
}
```

3. 服务端收到心跳数据后,需校验SN是否正确，并及时响应以下数据，以表示链路畅通
```JSON
Server -> Device
{
    "v":"0",
    "type": "pong", 
}
```
4. 后续客户端以每2min一次的频率向服务端发送心跳保持连接，服务端超过2min没有收到客户端的心跳数据，服务端会端掉连接，客户端发送心跳失败需重新建立连接


#### 2. 安全层
安全层对协议层原生数据进行对称加密,密钥由握手阶段通过非对称加密协商交换，得到输出流由传输层进行传输。暂不引入加密安全策略，因此该层目前不对协议层数据进行任何处理。

#### 3. 协议层
协议层定义数据传输格式，数据以JSON格式表达。协议层消息包含协议版本、消息类型、sn、与消息内容。其中消息类型包含`"get"`,`"set"`与`"event"`,分别表示服务端*获取*、*设置*与设备端*上报***端单元**状态(`status`)。

链接：

[端单元及其寻址](https://github.com/yulincoder/Controller-Dvice/blob/master/%E8%AE%BE%E5%A4%87%E6%8A%BD%E8%B1%A1%E6%A8%A1%E5%9E%8B.md)

具体协议格式如下:
1. 服务端获取端单元状态, `${unitid}`表示一个端单元地址,`v`标识CSoD协议版本号
```json
Server -> Device
get status mssage:
{
    "v":"0",
    "type": "get",
     "sn": "${sn}",                    
    "${unitid}": "",
    "${unitid}": "",
    ...
}
```
客户端响应端单元状态
```json
Device -> Server
get status mssage:
{
    "v":"0",
    "type": "getack",
     "sn": "${sn}",                    
    "${unitid}": "${status}",
    "${unitid}": "${status}",
    ...
}
```
可以看到，服务端get消息中，端单元地址key对应的value是无效的空字符串，而客户端响应消息中端单元地址对应的value是有效的

2. 服务端设置端单元状态, `${unitid}`表示一个端单元地址
```json
Server -> Device
get status mssage:
{
    "v":"0",
    "type": "set",
     "sn": "${sn}",                    
    "${unitid}": "${status}",
    "${unitid}": "${status}",
    ...
}
```
客户端响应端单元状态
```json
Device -> Server
get status mssage:
{
    "v":"0",
    "type": "setack",
     "sn": "${sn}",                    
    "${unitid}": "${status}",
    "${unitid}": "${status}",
    ...
}
```

3. 设备端主动上报端单元状态
```json
Device -> Server
get status mssage:
{
    "v":"0",
    "type": "event",
     "sn": "${sn}",                    
    "${unitid}": "${status}",
    "${unitid}": "${status}",
    ...
}
```
