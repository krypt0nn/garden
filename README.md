# 🏡 garden - a decentralized social platform built upon the [flowerpot](https://github.com/krypt0nn/flowerpot) project

- [garden-protocol](./garden-protocol) - rust type definitions for the garden
  protocol.
- [garden-client](./garden-client) - canonical protocol implementation in form
  of a GTK client application (linux only).

## Platform concept

Since this is an education project and it will never be used by anybody I won't
spend time and effort to make a "real" social network. Instead, garden-protocol
defines a very simple "post" entity and some related things.

### Posts

Each post has a content field, which is the text written by the post's author,
and a list of tags - short, strictly limited strings. Think about them as about
hashtags in other social networks: tags can be used to search for posts about
specific topics. For example, some sport-related post can be tagged with "sport"
tag, and if somebody wants to read about sport - they could easily find related
posts.

| Field     | Type     | Description                         |
| --------- | -------- | ----------------------------------- |
| `content` | `string` | Content of the post (user text)     |
| `tags`    | `tag[]`  | List of tags provided by the author |

Some tags have special meaning, and while this meaning is not implemented on the
protocol level it is implemented in the default garden application and is
recommended to be supported as well by all the other applications implementing
this protocol.

| Tag       | Meaning                                                                        |
| --------- | ------------------------------------------------------------------------------ |
| `nsfw`    | Not safe for work posts must be hidden by default                              |
| `spoiler` | Spoiler posts must be muted (their content must be invisible until clicked on) |

Currently posts' content must be limited by up to 8192 bytes, and they must have
only up to 20 tags.

### Comments

Users can add comments to the posts. Unlike posts, comments don't have the
`tags` field. Instead they have an `address` field which references the original
post.

| Field     | Type     | Description                        |
| --------- | -------- | ---------------------------------- |
| `content` | `string` | Content of the comment (user text) |
| `address` | `hash`   | Hash of flowerpot transaction      |

Posts can also be nested. You can provide another post's address instead of a
post to add a comment on existing comment.

Currently comments' content must be limited by up to 8192 bytes (so be equal
to the posts' content).

### Reactions

Users can add reactions to the posts. Reactions are pre-defined on the protocol
level and are connected to existing unicode emojis.

| Field      | Type     | Description                   |
| ---------- | -------- | ----------------------------- |
| `reaction` | `string` | Name of the reaction          |
| `address`  | `hash`   | Hash of flowerpot transaction |

List of currently available reactions:

| Name             | Emoji |
| ---------------- | ----- |
| `thumb_up`       | 👍    |
| `thumbs_down`    | 👎    |
| `heart`          | 🧡    |
| `broken_heart`   | 💔    |
| `hundred_points` | 💯    |
| `folded_hands`   | 🙏    |
| `party_popper`   | 🎉    |

No user-provided reactions will be supported. Only one reaction is allowed.
If multiple reactions are sent, then only the latest one is counted. If multiple
reactions are stored within the same block, then reaction with lower
transaction's hash is counted.

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
All the components are licensed under [GPL-3.0](LICENSE)
