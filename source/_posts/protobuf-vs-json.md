---
title: Protobuf VS Json
date: 2020-02-29 08:32:28
categories: protobuf
tags: protobuf,json
---

在分布式应用或者微服务架构中，各个服务之间通常使用json或者xml结构数据进行通信，通常情况下，是没什么问题的，但是在高性能和大数据通信的系统当中，如果有办法可以压缩数据量，提高传输效率，显然会给用户带来更快更流畅的体验。
google公司就通过使用一种新的数据交换格式办到了这点，新的数据交换的格式叫做protobuf。

protobuf有多屌呢，可以看一下下面的官方测试报告：

![](https://ptechen.github.io/images/4328038-586af6b3fc3228af.png)
![](https://ptechen.github.io/images/4328038-4c387af58311cd9c.png)