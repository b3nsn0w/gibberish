Turn files into gibberish, then de-gibberish them to get back the original file.
Also known as encryption, but we don't want to get pretentious here, plus
gibberish isn't really secret if you don't set a password.

# Motivation

Gibberish is intended to be a clean and simple way of exchanging files across
automated filters. Say, you wanted to send a `meme.gif` file to someone, but the
chat app you use disallows gif files for some weird reason, or processes them
and destroys the quality in the process. Perhaps you want to send a `photos.rar`
archive of the photos from your last trip, but the app disallows rar files in
fear of someone putting a virus in them. We've all seen these filters, and while
they clearly have some use, they often go way too far and are a hassle.

With gibberish, you can easily solve this issue. You can just turn a file into
gibberish, and then your recipient can de-gibberish the file. Simple, clean,
easy. No need for password-locked rar files or anything crazy and complex.

# Installation

If you got [the Rust tools](https://rustup.rs/) installed, you can use Cargo:

    cargo install gibberish

If you don't, you'll have to wait a bit. Gibberish is in development, there are
no precompiled binaries just yet.

# Usage

Engibberish files with the `gibberish` command:

    $ gibberish file.png
    Gibberish written to file.gibberish

(Is 'engibberish' this even a word? Well, now it certainly is...)

Then de-gibberish them with `gibberish -d`:

    $ gibberish -d file.gibberish
    Decoded gibberish to file.png

The gibberish file format stores the extension. You can use it for any kind of
file, all of them will be encoded to an extension of your choice (defaults to
'gibberish'), and decoded to the original extension.

# Advanced Usage

## Password Locking

You can set a password on your gibberish:

    $ gibberish -p secret.png
    Passphrase:
    Confirm passphrase:
    Gibberish written to secret.gibberish
    
    $ gibberish -d -p secret.gibberish
    Passphrase:
    Decoded gibberish to secret.png

This gives you similar-looking gibberish, but it will be encrypted with the
password you provided. The encryption standard is NaCl's Secretbox
(xsalsa20poly1305), with the password derived using libsodium's default password
hasher (scryptsalsa208sha256). In layman's terms, it's strong enough that your
password is going to be the weakest point.

## File Extension

You can set any extension to your output gibberish:

    $ gibberish -e docx hidden.png
    Gibberish written to hidden.docx

    $ gibberish -d hidden.docx
    Decoded gibberish to hidden.png

It's important to note that all gibberish files are encrypted, but if you don't
provide one, **the extension of the output file is the password**. In other
words, `file.gibberish` was encrypted with "gibberish" as the password, and
`hidden.docx` uses "docx" (unless you set a password yourself).

This also means that if you rename `hidden.docx` to, say, `hidden.xlsx`, it will
fail to de-gibberish, because it will try to use "xlsx" as the password while it
was gibberished with "docx".

    $ gibberish -e docx hidden.png
    Gibberish written to hidden.docx

    $ mv hidden.docx hidden.xlsx

    $ gibberish -d hidden.xlsx
    Error: failed to decode gibberish

In this case, just enter the original extension ("docx" in this case) as the
password:

    $ gibberish -d -p hidden.xlsx
    Passphrase:
    Decoded gibberish to hidden.png

# File format

The gibberish file consists of three concatenated binary fields:

  Name  |    Length    |           Description
------- | ------------ | --------------------------------
Salt    | 32 bytes     | The salt for password hashing
Nonce   | 24 bytes     | The nonce for the secretbox
Content | rest of file | A secretbox, containing the file

Inside the secretbox, there is a [MessagePack](https://msgpack.org/) object in
the following layout:

    Map {
      "extension": String,
      "file": Binary
    }

Where the value of the "extension" field is the original file extension, and the
value associated to the "file" key is the content of the original file.

## Filter Avoidance

While simple in format, this file is very hard to filter.

  - The file itself looks completely random. The content field looks random, and
    the other two fields are actually random. There is no magic number, no
    public metadata, nothing to distinguish a gibberish from a random binary.
    You could say it looks like gibberish.
  - Any file, with any extension can be gibberish, it is not limited to
    `*.gibberish`.
  - Even if the key is public (the gibberish has no password set to it and has
    not been renamed) you have to hash the entire file to detect gibberish.
  - Even if you do the above check, users can easily rename the file, use a
    different extension, or set a password, rendering your filter useless.

The point of gibberish is that you have to follow _and understand_ the
conversation to tell it apart from randomness, and that's something only humans
can do. For now, at least.

If you're a user of gibberish, reading this, just set a password and tell your
recipient what it is.

# TODO List

While the core algorithm is ready, gibberish still needs significant quality of
life improvements before it can be ready for its job, including:

  - A simple, friendly user interface
  - Precompiled binaries because no one's gonna install rustup
  - Shell integration on Windows (and maybe Linux)
  - Mac support? If you develop for those things, PRs are welcome
  - Mobile apps (figure out how the UX should even look, in the first place)
  - A website, maybe (low priority)

As you can see, gibberish is barely anything more than a proof of concept at
this stage.

# Contributing

Pull requests, issues, and ideas are welcome, feel free use github like it was
reddit. Just a few rules: don't do anything illegal and respect each other.

Gibberish is available under the MIT license.
