global handleNopInterrupt

section .text
handleNopInterrupt:
	push rax
	mov ax, 0x20
	out 0x20, ax
	pop rax
	iretq
