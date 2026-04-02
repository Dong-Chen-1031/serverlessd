# serverlessd
Serverless workers management architecture. **Work in progress**.
The publicity is just for quicker sharing.

**Unimplemented**:

To start a worker locally:

```sh
$ serverlessd one my-worker.js
```

To start a whole system for managing multiple workers:

```rs
$ serverlessd start --threads 10
```

***

## 介紹
一個基於 V8 的 Serverless Runtime，目標相容 Cloudflare Workers，但加了一些自訂義功能。

這個專案讓你可以在不需要管理伺服器的情況下執行 JavaScript。

你只需要寫一段程式：
- 有請求進來時就執行
- 自動擴展
- 用完就結束

概念類似 [Cloudflare Workers](https://developers.cloudflare.com/workers)。

好，其實基本上我也不知道我在做啥。

***

[社展](https://www.instagram.com/ckefgisc_latent_2026)
