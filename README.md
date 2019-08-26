# piet-dx12
Experimental GPU-based 2D renderer using DX12.

# Features

* GPU based rendering of large number of 2D objects is optimized using tiles to distribute workload on GPU processors efficiently. Workload distribution and object rendering are both achieved using compute kernels. 
* Basic text rendering using glyph atlases.
* Basic implementation of [piet](https://github.com/linebender/piet) specification.

# Future Work

* Implement basic path rendering using ideas in [RAVG](http://hhoppe.com/ravg.pdf).
* Implement additional primitives of interest in UIs: e.g. rounded rects.
* Use [druid-shell](https://github.com/xi-editor/druid/tree/master/druid-shell) for window management, with [smooth window resizing](https://raphlinus.github.io/rust/gui/2019/06/21/smooth-resize-test.html).
* Integrate properly into piet ecosystem using [`piet-common`](https://github.com/linebender/piet/tree/master/piet-common).
  
