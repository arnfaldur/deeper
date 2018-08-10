package main

func parts(c complex128) (float64, float64) {
	return real(c), imag(c)
}

func apply(c complex128, i func(complex128, complex128) complex128, f float64) complex128 {
	return i(c, complex(f, 0))
}

func div(c complex128, f float64) complex128 {
	return c / complex(f, 0)
}

func cmul(c complex128, f float64) complex128 {
	return c * complex(f, 0)
}
