; =============================================================================
; Program: Hello World
; Description: A simple MASM program using Irvine32 library
; =============================================================================

INCLUDE Irvine32.inc

.data
    message BYTE "Hello, World!", 0

.code
main PROC
    ; Display the message
    mov  edx, OFFSET message
    call WriteString
    call Crlf

    ; Exit program
    exit
main ENDP

END main