# Hermitian matrix

In mathematics, a **Hermitian matrix** (or **self-adjoint matrix**) is a [complex](https://example.com/wiki/Complex_number "Complex number") [square matrix](https://example.com/wiki/Square_matrix "Square matrix") that is equal to its own [conjugate transpose](https://example.com/wiki/Conjugate_transpose "Conjugate transpose")—that is, the element in the i-th row and j-th column is equal to the [complex conjugate](https://example.com/wiki/Complex_conjugate "Complex conjugate") of the element in the j-th row and i-th column, for all indices i and j: 

or in matrix form: 

. 

Hermitian matrices can be understood as the complex extension of real [symmetric matrices](https://example.com/wiki/Symmetric_matrix "Symmetric matrix"). 

If the [conjugate transpose](https://example.com/wiki/Conjugate_transpose "Conjugate transpose") of a matrix is denoted by , then the Hermitian property can be written concisely as 

Hermitian matrices are named after [Charles Hermite](https://example.com/wiki/Charles_Hermite "Charles Hermite"), who demonstrated in 1855 that matrices of this form share a property with real symmetric matrices of always having real [eigenvalues](https://example.com/wiki/Eigenvalues_and_eigenvectors "Eigenvalues and eigenvectors"). Other, equivalent notations in common use are , although note that in [quantum mechanics](https://example.com/wiki/Quantum_mechanics "Quantum mechanics"), typically means the [complex conjugate](https://example.com/wiki/Complex_conjugate "Complex conjugate") only, and not the [conjugate transpose](https://example.com/wiki/Conjugate_transpose "Conjugate transpose"). 

## Alternative characterizations

Hermitian matrices can be characterized in a number of equivalent ways, some of which are listed below: 

### Equality with the adjoint

A square matrix is Hermitian if and only if it is equal to its [adjoint](https://example.com/wiki/Hermitian_adjoint "Hermitian adjoint"), that is, it satisfies 

for any pair of vectors , where denotes [the inner product](https://example.com/wiki/Dot_product "Dot product") operation.

This is also the way that the more general concept of [self-adjoint operator](https://example.com/wiki/Self-adjoint_operator "Self-adjoint operator") is defined. 

### Reality of quadratic forms

A square matrix is Hermitian if and only if it is such that 

### Spectral properties

A square matrix is Hermitian if and only if it is unitarily [diagonalizable](https://example.com/wiki/Diagonalizable_matrix "Diagonalizable matrix") with real [eigenvalues](https://example.com/wiki/Eigenvalues_and_eigenvectors "Eigenvalues and eigenvectors"). 

## Applications

Hermitian matrices are fundamental to the quantum theory of [matrix mechanics](https://example.com/wiki/Matrix_mechanics "Matrix mechanics") created by [Werner Heisenberg](https://example.com/wiki/Werner_Heisenberg "Werner Heisenberg"), [Max Born](https://example.com/wiki/Max_Born "Max Born"), and [Pascual Jordan](https://example.com/wiki/Pascual_Jordan "Pascual Jordan") in 1925. 

## Examples

In this section, the conjugate transpose of matrix is denoted as , the transpose of matrix is denoted as and conjugate of matrix is denoted as . 

See the following example: 

The diagonal elements must be [real](https://example.com/wiki/Real_number "Real number"), as they must be their own complex conjugate. 

Well-known families of [Pauli matrices](https://example.com/wiki/Pauli_matrices "Pauli matrices"), [Gell-Mann matrices](https://example.com/wiki/Gell-Mann_matrices "Gell-Mann matrices") and their generalizations are Hermitian. In [theoretical physics](https://example.com/wiki/Theoretical_physics "Theoretical physics") such Hermitian matrices are often multiplied by [imaginary](https://example.com/wiki/Imaginary_number "Imaginary number") coefficients,[^1][^2] which results in *skew-Hermitian* matrices (see [below](https://example.com/mozilla--wikipedia-3#facts)). 

Here, we offer another useful Hermitian matrix using an abstract example. If a square matrix equals the [multiplication of a matrix](https://example.com/wiki/Matrix_multiplication "Matrix multiplication") and its conjugate transpose, that is, , then is a Hermitian [positive semi-definite matrix](https://example.com/wiki/Positive_semi-definite_matrix "Positive semi-definite matrix"). Furthermore, if is row full-rank, then is positive definite. 

## Properties

*   The entries on the [main diagonal](https://example.com/wiki/Main_diagonal "Main diagonal") (top left to bottom right) of any Hermitian matrix are [real](https://example.com/wiki/Real_number "Real number").

*Proof:* By definition of the Hermitian matrix

so for *i* = *j* the above follows. 

Only the [main diagonal](https://example.com/wiki/Main_diagonal "Main diagonal") entries are necessarily real; Hermitian matrices can have arbitrary complex-valued entries in their [off-diagonal elements](https://example.com/wiki/Off-diagonal_element "Off-diagonal element"), as long as diagonally-opposite entries are complex conjugates. 

*   A matrix that has only real entries is Hermitian [if and only if](https://example.com/wiki/If_and_only_if "If and only if") it is [symmetric](https://example.com/wiki/Symmetric_matrix "Symmetric matrix"). A real and symmetric matrix is simply a special case of a Hermitian matrix.

*Proof:* by definition. Thus H*ij* = H*ji* (matrix symmetry) if and only if (H*ij* is real). 

*   Every Hermitian matrix is a [normal matrix](https://example.com/wiki/Normal_matrix "Normal matrix"). That is to say, AAH = AHA.

*Proof:* A = AH, so AAH = AA = AHA. 

*   The finite-dimensional [spectral theorem](https://example.com/wiki/Spectral_theorem "Spectral theorem") says that any Hermitian matrix can be [diagonalized](https://example.com/wiki/Diagonalizable_matrix "Diagonalizable matrix") by a [unitary matrix](https://example.com/wiki/Unitary_matrix "Unitary matrix"), and that the resulting diagonal matrix has only real entries. This implies that all [eigenvalues](https://example.com/wiki/Eigenvectors "Eigenvectors") of a Hermitian matrix A with dimension n are real, and that A has n linearly independent [eigenvectors](https://example.com/wiki/Eigenvector "Eigenvector"). Moreover, a Hermitian matrix has [orthogonal](https://example.com/wiki/Orthogonal "Orthogonal") eigenvectors for distinct eigenvalues. Even if there are degenerate eigenvalues, it is always possible to find an [orthogonal basis](https://example.com/wiki/Orthogonal_basis "Orthogonal basis") of ℂ*n* consisting of n eigenvectors of A.

*   The sum of any two Hermitian matrices is Hermitian.

*Proof:* as claimed. 

*   The [inverse](https://example.com/wiki/Inverse_matrix "Inverse matrix") of an invertible Hermitian matrix is Hermitian as well.

*Proof:* If , then , so as claimed. 

*   The [product](https://example.com/wiki/Matrix_multiplication "Matrix multiplication") of two Hermitian matrices A and B is Hermitian if and only if *AB* = *BA*.

*Proof:* Note that Thus [if and only if](https://example.com/wiki/If_and_only_if "If and only if") . 

Thus *A**n* is Hermitian if A is Hermitian and n is an integer. 

*   For an arbitrary complex valued vector v the product is real because of . This is especially important in quantum physics where Hermitian matrices are operators that measure properties of a system e.g. total [spin](https://example.com/wiki/Spin_\(physics\) "Spin (physics)") which have to be real.

*   The Hermitian complex n-by-n matrices do not form a [vector space](https://example.com/wiki/Vector_space "Vector space") over the [complex numbers](https://example.com/wiki/Complex_number "Complex number"), ℂ, since the identity matrix *I**n* is Hermitian, but *i* *I**n* is not. However the complex Hermitian matrices *do* form a vector space over the [real numbers](https://example.com/wiki/Real_numbers "Real numbers") ℝ. In the 2*n*2-[dimensional](https://example.com/wiki/Dimension_of_a_vector_space "Dimension of a vector space") vector space of complex *n* × *n* matrices over ℝ, the complex Hermitian matrices form a subspace of dimension *n*2. If *E**jk* denotes the n-by-n matrix with a 1 in the *j*,*k* position and zeros elsewhere, a basis (orthonormal w.r.t. the Frobenius inner product) can be described as follows:

together with the set of matrices of the form 

and the matrices 

where denotes the complex number , called the *[imaginary unit](https://example.com/wiki/Imaginary_unit "Imaginary unit")*. 

*   If n orthonormal eigenvectors of a Hermitian matrix are chosen and written as the columns of the matrix U, then one [eigendecomposition](https://example.com/wiki/Eigendecomposition_of_a_matrix "Eigendecomposition of a matrix") of A is where and therefore

where are the eigenvalues on the diagonal of the diagonal matrix . 

*   The determinant of a Hermitian matrix is real:

*Proof:* 

Therefore if . 

(Alternatively, the determinant is the product of the matrix's eigenvalues, and as mentioned before, the eigenvalues of a Hermitian matrix are real.) 

## Decomposition into Hermitian and skew-Hermitian

Additional facts related to Hermitian matrices include: 

*   The sum of a square matrix and its conjugate transpose is Hermitian.

*   The difference of a square matrix and its conjugate transpose is [skew-Hermitian](https://example.com/wiki/Skew-Hermitian_matrix "Skew-Hermitian matrix") (also called antihermitian). This implies that the [commutator](https://example.com/wiki/Commutator "Commutator") of two Hermitian matrices is skew-Hermitian.

*   An arbitrary square matrix C can be written as the sum of a Hermitian matrix A and a skew-Hermitian matrix B. This is known as the Toeplitz decomposition of C.[^3]:p. 7

## Rayleigh quotient

In mathematics, for a given complex Hermitian matrix *M* and nonzero vector *x*, the Rayleigh quotient[^4] , is defined as:[^3]:p. 234[^5] 

. 

For real matrices and vectors, the condition of being Hermitian reduces to that of being symmetric, and the conjugate transpose to the usual transpose . Note that for any non-zero real scalar . Also, recall that a Hermitian (or real symmetric) matrix has real eigenvalues. 

It can be shown that, for a given matrix, the Rayleigh quotient reaches its minimum value (the smallest eigenvalue of M) when is (the corresponding eigenvector). Similarly, and . 

The Rayleigh quotient is used in the min-max theorem to get exact values of all eigenvalues. It is also used in eigenvalue algorithms to obtain an eigenvalue approximation from an eigenvector approximation. Specifically, this is the basis for Rayleigh quotient iteration. 

The range of the Rayleigh quotient (for matrix that is not necessarily Hermitian) is called a numerical range (or spectrum in functional analysis). When the matrix is Hermitian, the numerical range is equal to the spectral norm. Still in functional analysis, is known as the spectral radius. In the context of C\*-algebras or algebraic quantum mechanics, the function that to *M* associates the Rayleigh quotient *R*(*M*, *x*) for a fixed *x* and *M* varying through the algebra would be referred to as "vector state" of the algebra. 

## See also

*   [Vector space](https://example.com/wiki/Vector_space "Vector space")
*   [Skew-Hermitian matrix](https://example.com/wiki/Skew-Hermitian_matrix "Skew-Hermitian matrix") (anti-Hermitian matrix)
*   [Haynsworth inertia additivity formula](https://example.com/wiki/Haynsworth_inertia_additivity_formula "Haynsworth inertia additivity formula")
*   [Hermitian form](https://example.com/wiki/Hermitian_form "Hermitian form")
*   [Self-adjoint operator](https://example.com/wiki/Self-adjoint_operator "Self-adjoint operator")
*   [Unitary matrix](https://example.com/wiki/Unitary_matrix "Unitary matrix")

## References

## External links

*   [Hazewinkel, Michiel](https://example.com/wiki/Michiel_Hazewinkel "Michiel Hazewinkel"), ed. (2001) \[1994\], ["Hermitian matrix"](https://www.encyclopediaofmath.org/index.php?title=p/h047070), *[Encyclopedia of Mathematics](https://example.com/wiki/Encyclopedia_of_Mathematics "Encyclopedia of Mathematics")*, Springer Science+Business Media B.V. / Kluwer Academic Publishers, [ISBN](https://example.com/wiki/International_Standard_Book_Number "International Standard Book Number") [978-1-55608-010-4](https://example.com/wiki/Special:BookSources/978-1-55608-010-4 "Special:BookSources/978-1-55608-010-4")
*   [Visualizing Hermitian Matrix as An Ellipse with Dr. Geo](https://www.cyut.edu.tw/~ckhung/b/la/hermitian.en.php), by Chao-Kuei Hung from Chaoyang University, gives a more geometric explanation.
*   ["Hermitian Matrices"](http://www.mathpages.com/home/kmath306/kmath306.htm). *MathPages.com*.

[^1]: **[^](https://example.com/mozilla--wikipedia-3#cite_ref-1)** [Frankel, Theodore](https://example.com/wiki/Theodore_Frankel "Theodore Frankel") (2004). [*The Geometry of Physics: an introduction*](https://books.google.com/books?id=DUnjs6nEn8wC&lpg=PA652&dq=%22Lie%20algebra%22%20physics%20%22skew-Hermitian%22&pg=PA652#v=onepage&q&f=false). [Cambridge University Press](https://example.com/wiki/Cambridge_University_Press "Cambridge University Press"). p. 652. [ISBN](https://example.com/wiki/International_Standard_Book_Number "International Standard Book Number") [0-521-53927-7](https://example.com/wiki/Special:BookSources/0-521-53927-7 "Special:BookSources/0-521-53927-7").

[^2]: **[^](https://example.com/mozilla--wikipedia-3#cite_ref-2)** [Physics 125 Course Notes](http://www.hep.caltech.edu/~fcp/physics/quantumMechanics/angularMomentum/angularMomentum.pdf) at [California Institute of Technology](https://example.com/wiki/California_Institute_of_Technology "California Institute of Technology")

[^3]: ^ [***a***](https://example.com/mozilla--wikipedia-3#cite_ref-HornJohnson_3-0) [***b***](https://example.com/mozilla--wikipedia-3#cite_ref-HornJohnson_3-1) Horn, Roger A.; Johnson, Charles R. (2013). *Matrix Analysis, second edition*. Cambridge University Press. [ISBN](https://example.com/wiki/International_Standard_Book_Number "International Standard Book Number") [9780521839402](https://example.com/wiki/Special:BookSources/9780521839402 "Special:BookSources/9780521839402").

[^4]: **[^](https://example.com/mozilla--wikipedia-3#cite_ref-4)** Also known as the **Rayleigh–Ritz ratio**; named after [Walther Ritz](https://example.com/wiki/Walther_Ritz "Walther Ritz") and [Lord Rayleigh](https://example.com/wiki/Lord_Rayleigh "Lord Rayleigh").

[^5]: **[^](https://example.com/mozilla--wikipedia-3#cite_ref-5)** Parlet B. N. *The symmetric eigenvalue problem*, SIAM, Classics in Applied Mathematics,1998