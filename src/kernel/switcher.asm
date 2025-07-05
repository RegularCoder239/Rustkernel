.text
.code64
.global jump_state
.global load_state
load_state:
	mov %rax, 0x0(%rsi)
	mov %rbx, 0x8(%rsi)
	mov %rcx, 0x10(%rsi)
	mov %rdx, 0x18(%rsi)
	mov %rsi, 0x20(%rsi)
	mov %rdi, 0x28(%rsi)
	mov %rbp, 0x30(%rsi)
	mov %rsp, 0x38(%rsi)
	add $0x8, 0x38(%rsi)
	mov %r8, 0x40(%rsi)
	mov %r9, 0x48(%rsi)
	mov %r10, 0x50(%rsi)
	mov %r11, 0x58(%rsi)
	mov %r12, 0x60(%rsi)
	mov %r13, 0x68(%rsi)
	mov %r14, 0x70(%rsi)
	mov %r15, 0x78(%rsi)
	sub $0x100, %rsp
	mov (%rsp), %rax
	mov %rax, 0x80(%rsi)
	xor %rax, %rax
	pushf
	pop 0x88(%rsi)
	mov %cs, %ax
	mov %rax, 0x90(%rsi)
	mov %ds, %ax
	mov %rax, 0x98(%rsi)
	rdgsbase %rax
	mov %rax, 0xa0(%rsi)

jump_state:
	mov %rax, 0xa0(%rdi)
	wrgsbase %rax

	push 0x98(%rdi)
	push 0x38(%rdi)
	push 0x88(%rdi)
	push 0x90(%rdi)
	push 0x80(%rdi)

	mov 0x0(%rdi), %rax
	mov 0x8(%rdi), %rbx
	mov 0x10(%rdi), %rcx
	mov 0x18(%rdi), %rdx
	mov 0x20(%rdi), %rsi

	mov 0x30(%rdi), %rbp

	mov 0x40(%rdi), %r8
	mov 0x48(%rdi), %r9
	mov 0x50(%rdi), %r10
	mov 0x58(%rdi), %r11
	mov 0x60(%rdi), %r12
	mov 0x68(%rdi), %r13
	mov 0x70(%rdi), %r14
	mov 0x78(%rdi), %r15

	mov 0x28(%rdi), %rdi

	iretq
