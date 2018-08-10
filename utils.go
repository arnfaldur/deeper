package main

func parts(c complex128) (float64, float64) {
	return real(c), imag(c)
}

func div(c complex128, f float64) complex128 {
	return c / complex(f, 0)
}
