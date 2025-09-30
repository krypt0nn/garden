# garden-server HTTP API reference

Both GET and POST types of requests are supported, although it's generally
recommended to use POST when large amount of data is sent
(e.g. when creating new post).

Errors are returned as JSON arrays of the following format:

```ts
type ErrorResponse = {
    error: {
        // Standardized code of the error.
        code: string;

        // Non-standardized message with the error text.
        message: string;
    };
};
```

APIs which involve creation of new entities on the platform or their
modification will return signed transaction in the following format:

```ts
type Transaction = {
    hash: string;
    transaction: string;
};
```

You should use the network API to send these transactions and monitor their
status.

## Keypair API

Keypair consist of two keys - a verifying key, which is an alternative to the
"user login", and a signing key, which is an alternative to the "user password".
Except you can derive the user's "login" from its "password". This means that
the "password" is highly secret and it should not be leaked at any cost.
Moreover, users are not allowed to change this "password" because it means they
would need to change the "login" too, because they're connected, which means
making a new account! So, if you don't want to lose your account - never ever
leak your signing key!

This API allows you to work with keypairs - create new one (basically make an
account) or derive verifying key from a signing key.

### `/api/v1/keypair/create`

Create new signing and verifying key pair. You can specify a seed phrase which
will be used for better RNG properties.

| Param  | Required | Value                |
| ------ | -------- | -------------------- |
| `seed` | No       | Any phrase or number |

```ts
type Response = {
    signing_key: string,
    verifying_key: string
};
```

### `/api/v1/keypair/export`

Obtain verifying key from the provided signing key.

| Param         | Required | Value       |
| ------------- | -------- | ----------- |
| `signing_key` | Yes      | Signing key |

```ts
type Response = {
    verifying_key: string
};
```

Possible errors:

| Error code                  | Explanation                       |
| --------------------------- | --------------------------------- |
| `missing_required_param`    | Missing required parameter input  |
| `keypair_deserialize_error` | Failed to deserialize signing key |

## Names API

Since users' addresses are their verifying keys, which are 32 bytes long, or
about 40 characters of base64 string long, nobody ever would remember them.
It is much more convenient to have a table of short English text names and their
connected verifying keys.

This API allows you to register new names or query existing ones.

### `/api/v1/name/register`

Try to register new unique name.

| Param         | Required | Value                                       |
| ------------- | -------- | ------------------------------------------- |
| `signing_key` | Yes      | Signing key which will sign the transaction |
| `name`        | Yes      | Unique name to associate verifying key with |

Verifying key which will be associated with provided name is derived from the
signing key to prevent potential abuses of the API.

```ts
type Response = Transaction;
```

Possible errors:

| Error code                  | Explanation                                |
| --------------------------- | ------------------------------------------ |
| `missing_required_param`    | Missing required parameter input           |
| `keypair_deserialize_error` | Failed to deserialize a key                |
| `invalid_name`              | Name doesn't follow allowed format         |
| `name_already_taken`        | Provided name is already taken             |
| `signing_error`             | Failed to sign transaction                 |
| `transaction_not_accepted`  | Transaction is not accepted by the network |

### `/api/v1/name/query`

Query names.

| Param           | Required | Value         |
| --------------- | -------- | ------------- |
| `name`          | No       | Name          |
| `verifying_key` | No       | Verifying key |

The method will return all the records with partially or fully matched names
or fully matched verifying key.

```ts
type Record = {
    name: string;
    verifying_key: string;
};

type Response = Record[];
```

Possible errors:

| Error code                  | Explanation                      |
| --------------------------- | -------------------------------- |
| `missing_required_param`    | Missing required parameter input |
| `keypair_deserialize_error` | Failed to deserialize a key      |

## Posts API

Post is the main communication object of the protocol. A post can reference
another post so function as a reply or used for hierarchical comment section.

This API allows you to create or query posts made by other users.

### `/api/v1/post/create`

Create new post.

| Param         | Required | Value                                       |
| ------------- | -------- | ------------------------------------------- |
| `signing_key` | Yes      | Signing key which will sign the transaction |
| `content`     | Yes      | Text of the post                            |
| `reply_to`    | No       | Address of another post                     |

```ts
type Response = Transaction;
```

Possible errors:

| Error code                  | Explanation                      |
| --------------------------- | -------------------------------- |
| `missing_required_param`    | Missing required parameter input |
| `keypair_deserialize_error` | Failed to deserialize a key      |
| `signing_error`             | Failed to sign transaction       |

### `/api/v1/post/query`

Query posts.

| Param     | Required | Value                            |
| --------- | -------- | -------------------------------- |
| `from`    | No       | Verifying key of the post author |

...

## Transactions API

All your actions on the platform are called "events" and these events are
encoded in a special format and exchanged on the decentralized flowerpot
network as transactions within a blockchain.

This API allows you to send transactions created by other APIs and monitor their
status (some transactions can be rejected by the network, some could require
long time to be accepted, etc.).

### `/api/v1/transaction/send`

Try to send transaction to the network.

| Param         | Required | Value                              |
| ------------- | -------- | ---------------------------------- |
| `transaction` | Yes      | Transaction to send to the network |

```ts
type Response = Transaction;
```

Possible errors:

| Error code                 | Explanation                                |
| -------------------------- | ------------------------------------------ |
| `missing_required_param`   | Missing required parameter input           |
| `invalid_transaction`      | Invalid transaction format                 |
| `transaction_not_accepted` | Transaction is not accepted by the network |

### `/api/v1/transaction/status`

Get status of a transaction.

| Param  | Required | Value               |
| ------ | -------- | ------------------- |
| `hash` | Yes      | Hash of transaction |

```ts
type NotAvailable = {
    status: 'not_available';
}

type Pending = {
    status: 'pending';
};

type Staged = {
    status: 'staged';
    block: {
        hash: string;
        timestamp: number;
    };
};

type Response = NotAvailable | Pending | Staged;
```

Possible errors:

| Error code               | Explanation                      |
| ------------------------ | -------------------------------- |
| `missing_required_param` | Missing required parameter input |
| `hash_deserialize_error` | Failed to deserialize hash       |
