package main

import (
	"crypto/ed25519"
	"fmt"
)

func main() {
	b := []byte{108, 75, 154, 1, 135, 158, 88, 246, 92, 77, 139, 103, 47, 229, 239, 40, 220, 185, 84, 75, 117, 203, 247, 26, 91, 7, 240, 156, 134, 212, 162, 234}
	fmt.Println(ed25519.NewKeyFromSeed(b))
}
