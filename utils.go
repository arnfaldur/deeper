package main

import "math/cmplx"

func parts(c complex128) (float64, float64) {
	return real(c), imag(c)
}

func div(c complex128, f float64) complex128 {
	return c / complex(f, 0)
}

func cmplxNorm(c complex128) complex128 {
	return c / complex(cmplx.Abs(c), 0)
}
