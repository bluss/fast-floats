fast-floats
===========

Please read the `API documentation on docs.rs`__

__ https://docs.rs/fast-floats/

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/bluss/fast-floats.svg?branch=master
.. _build_status: https://travis-ci.org/bluss/fast-floats

.. |crates| image:: http://meritbadge.herokuapp.com/fast-floats
.. _crates: https://crates.io/crates/fast-floats


Recent Changes
--------------

- 0.2.0

  - ``Fast`` now requires ``unsafe`` to create values: to be clear that it is a
    soundness issue to have problematic values in ``Fast``, i.e. non-finite
    floats. No safe constructors also means that ``Default`` and ``Zero`` traits disappear.
  - Fix bug in assign ops: they were previously implemented incorrectly (Except +=)

- 0.1.2

  - Use repr(transparent) on Fast
  - Add type aliases FF64, FF32 and a few trait impls

- 0.1.1

  - Add mixed operations (Fast<f64> + f64 etc.)

- 0.1.0

  - Initial release.


License
=======

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.


