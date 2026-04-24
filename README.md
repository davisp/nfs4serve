# nfs4serve - An NFSv4.1 Server Framework

This is an attempt at creating an NFSv4.1 server. Currently this is a rewrite
of the [nfsserve](https://github.com/huggingface/nfsserve) crate. I'm mostly
working on learning the underlying protocol bits and to do that I find it
easier to type things out as I explore the basics of the protocol. I'm open to
eventually contributing this work back to the upstream crate if I don't diverge
too far from their design. However, from what I've read so far, the v3 vs v4.1
changes might be too drastic to make it worth combining implementations.

Regardless, this is a big shoutout to the nfsserve authors. Without their
prior art, this would be taking me significantly more effort translating the
old RFCs into code.
