# hiirc
**hiirc** is a **hi**gh level **irc** client library. The primary goal is for it to be easy to use and featureful.
It's build on top of [loirc](https://github.com/sbstp/loirc) and offers the same stability and reliability but
with a much friendlier and abstract interface.

You can see the difference between loirc and hiirc by checking the peekaboo examples.
([loirc](https://github.com/sbstp/loirc/blob/master/examples/peekaboo.rs)/
[hiirc](https://github.com/sbstp/hiirc/blob/master/examples/peekaboo.rs))

You can setup a client by implementing the [`Listener`](http://sbstp.github.io/hiirc/hiirc/trait.Listener.html) trait,
and giving an instance of your object to the [`dispatch`](http://sbstp.github.io/hiirc/hiirc/fn.dispatch.html) method
with [`Settings`](http://sbstp.github.io/hiirc/hiirc/struct.Settings.html) configured to your needs.

Just like **loirc**, server side is not a goal at the moment.

## Features
* event based API
* channel, nickname and topic collection
* various methods to send messages

## License
zlib license, see [LICENSE](LICENSE).
