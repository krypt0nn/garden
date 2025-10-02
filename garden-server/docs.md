# garden-server HTTP API reference

| Method                                | Action                            |
| ------------------------------------- | --------------------------------- |
| `GET /api/v1/account/new`             | Create new account                |
| `POST /api/v1/post`                   | Create new post                   |
| `GET /api/v1/post/<address>`          | Get info about a post             |
| `POST /api/v1/comment`                | Create new comment                |
| `GET /api/v1/comment/<address>`       | Get info about a comment          |
| `POST /api/v1/reaction`               | Add reaction to a post or comment |

## `GET /api/v1/account/new`

Create new garden-protocol account (signing and verifying keys pair).

### Request

No request fields needed.

### Response (200 OK)

```json
{
    "type": "object",
    "parameters": {
        "signing_key": {
            "type": "string",
            "description": "flowerpot signing key"
        },
        "verifying_key": {
            "type": "string",
            "description": "flowerpot verifying key"
        }
    },
    "required": [
        "signing_key",
        "verifying_key"
    ]
}
```

## `POST /api/v1/post`

Create new post and send it to the network.

### Request

```json
{
    "type": "object",
    "properties": {
        "signing_key": {
            "type": "string",
            "description": "flowerpot signing key"
        },
        "content": {
            "type": "string",
            "description": "content of the post"
        },
        "tags": {
            "type": "array",
            "items": {
                "type": "string"
            },
            "description": "tags of the post"
        }
    },
    "required": [
        "signing_key",
        "content",
        "tags"
    ]
}
```

### Response (200 OK)

```json
{
    "type": "object",
    "properties": {
        "transaction": {
            "type": "string",
            "description": "hash of created transaction"
        }
    },
    "required": [
        "transaction"
    ]
}
```

## `GET /api/v1/post/<address>`

Get information about a post. The address is a hash of the post's flowerpot
transaction.

### Response (200 OK)

```json
{
    "type": "object",
    "properties": {
        "status": {
            "type": "string",
            "description": "post transaction status",
            "enum": [
                "pending",
                "staged"
            ]
        },
        "content": {
            "type": "string",
            "description": "content of the post"
        },
        "tags": {
            "type": "array",
            "items": {
                "type": "string"
            },
            "description": "tags of the post"
        },
        "comments": {
            "type": "array",
            "items": {
                "type": "string"
            },
            "description": "list of transactions' hashes of the direct post comments"
        },
        "reactions": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "name of the reaction"
                    },
                    "author": {
                        "type": "string",
                        "description": "flowerpot verifying key of the person who added this reaction"
                    }
                },
                "required": [
                    "name",
                    "author"
                ]
            }
        }
    },
    "required": [
        "status",
        "content",
        "tags",
        "comments",
        "reactions"
    ]
}
```

## `POST /api/v1/comment`

Create new comment and send it to the network.

### Request

```json
{
    "type": "object",
    "properties": {
        "signing_key": {
            "type": "string",
            "description": "flowerpot signing key"
        },
        "content": {
            "type": "string",
            "description": "content of the post"
        },
        "address": {
            "type": "string",
            "description": "hash of the post's or comment's transaction to reply to"
        }
    },
    "required": [
        "signing_key",
        "content",
        "address"
    ]
}
```

### Response (200 OK)

No data returned.

## `GET /api/v1/comment/<address>`

Get information about a comment. The address is a hash of the comment's
flowerpot transaction.

### Response (200 OK)

```json
{
    "type": "object",
    "properties": {
        "status": {
            "type": "string",
            "description": "comment transaction status",
            "enum": [
                "pending",
                "staged"
            ]
        },
        "content": {
            "type": "string",
            "description": "content of the post"
        },
        "original_address": {
            "type": "string",
            "description": "transaction address this comment is referenced to"
        },
        "comments": {
            "type": "array",
            "items": {
                "type": "string"
            },
            "description": "list of transactions' hashes of the direct comments"
        },
        "reactions": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "name of the reaction"
                    },
                    "author": {
                        "type": "string",
                        "description": "flowerpot verifying key of the person who added this reaction"
                    }
                },
                "required": [
                    "name",
                    "author"
                ]
            }
        }
    },
    "required": [
        "status",
        "content",
        "original_address",
        "comments",
        "reactions"
    ]
}
```

## `POST /api/v1/reaction`

Add new reaction to a post or a comment and send it to the network.

### Request

```json
{
    "type": "object",
    "properties": {
        "signing_key": {
            "type": "string",
            "description": "flowerpot signing key"
        },
        "name": {
            "type": "string",
            "description": "name of the reaction"
        },
        "address": {
            "type": "string",
            "description": "transaction address of a post or a comment"
        }
    },
    "required": [
        "signing_key",
        "name",
        "address"
    ]
}
```

### Response (200 OK)

No data returned.
