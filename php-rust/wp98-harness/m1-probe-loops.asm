# Sonda M1 build 2 (pressione, jump table) — loop caldi delle due forme
# FORMA A (preambolo di oggi): back-edge 0x1000010cc
1000010a8: f85903a8    	ldur	x8, [x29, #-0x70]
1000010ac: f90007e8    	str	x8, [sp, #0x8]
1000010b0: 1400000a    	b	0x1000010d8 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0x804>
1000010b4: f8570288    	ldur	x8, [x20, #-0x90]
1000010b8: f8337918    	str	x24, [x8, x19, lsl #3]
1000010bc: 91000668    	add	x8, x19, #0x1
1000010c0: f8178288    	stur	x8, [x20, #-0x88]
1000010c4: b00001d8    	adrp	x24, 0x10003a000 <dyld_stub_binder+0x10003a000>
1000010c8: 91186318    	add	x24, x24, #0x618
1000010cc: 528c3508    	mov	w8, #0x61a8             ; =25000
1000010d0: eb0802ff    	cmp	x23, x8
1000010d4: 54fff128    	b.hi	0x100000ef8 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0x624>
1000010d8: b40032b7    	cbz	x23, 0x10000172c <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xe58>
1000010dc: f94006a8    	ldr	x8, [x21, #0x8]
1000010e0: 52801609    	mov	w9, #0xb0               ; =176
1000010e4: 9b0922f4    	madd	x20, x23, x9, x8
1000010e8: a9780288    	ldp	x8, x0, [x20, #-0x80]
1000010ec: f9400901    	ldr	x1, [x8, #0x10]
1000010f0: eb01001f    	cmp	x0, x1
1000010f4: 54003162    	b.hs	0x100001720 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xe4c>
1000010f8: f9400508    	ldr	x8, [x8, #0x8]
1000010fc: 9b1c2008    	madd	x8, x0, x28, x8
100001100: 91000409    	add	x9, x0, #0x1
100001104: f8188289    	stur	x9, [x20, #-0x78]
100001108: 79400109    	ldrh	w9, [x8]
10000110c: 1000000a    	adr	x10, 0x10000110c <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0x838>
100001110: b8a97b0b    	ldrsw	x11, [x24, x9, lsl #2]
100001114: 8b0b014a    	add	x10, x10, x11
100001118: d61f0140    	br	x10

# FORMA B (split-borrow H-B1): back-edge 0x10000138c — frame TENUTO in x20, niente guardia/reload
100001354: b4002238    	cbz	x24, 0x100001798 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xec4>
100001358: f94006c8    	ldr	x8, [x22, #0x8]
10000135c: 52801609    	mov	w9, #0xb0               ; =176
100001360: 9b092314    	madd	x20, x24, x9, x8
100001364: a9780288    	ldp	x8, x0, [x20, #-0x80]
100001368: f9400901    	ldr	x1, [x8, #0x10]
10000136c: eb01001f    	cmp	x0, x1
100001370: 54001d22    	b.hs	0x100001714 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xe40>
100001374: d102a297    	sub	x23, x20, #0xa8
100001378: 14000009    	b	0x10000139c <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xac8>
10000137c: f8570288    	ldur	x8, [x20, #-0x90]
100001380: f8337915    	str	x21, [x8, x19, lsl #3]
100001384: 91000668    	add	x8, x19, #0x1
100001388: f8178288    	stur	x8, [x20, #-0x88]
10000138c: a9780288    	ldp	x8, x0, [x20, #-0x80]
100001390: f9400901    	ldr	x1, [x8, #0x10]
100001394: eb01001f    	cmp	x0, x1
100001398: 54001be2    	b.hs	0x100001714 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xe40>
10000139c: f9400508    	ldr	x8, [x8, #0x8]
1000013a0: 9b1c2008    	madd	x8, x0, x28, x8
1000013a4: 91000409    	add	x9, x0, #0x1
1000013a8: f8188289    	stur	x9, [x20, #-0x78]
1000013ac: 79400109    	ldrh	w9, [x8]
1000013b0: 1000000a    	adr	x10, 0x1000013b0 <__ZN8m1_probe4main17hfc47cdd84d5aaa01E+0xadc>
1000013b4: b8a97b4b    	ldrsw	x11, [x26, x9, lsl #2]
1000013b8: 8b0b014a    	add	x10, x10, x11
1000013bc: d61f0140    	br	x10
