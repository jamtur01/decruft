## Scaled Dot-Product Attention

We call the attention function on a set of queries simultaneously.

 Attention(Q,K,V)=softmax(QKTdk)V \\mathrm{Attention}(Q,K,V)=\\mathrm{softmax}(\\frac{QK^{T}}{\\sqrt{d\_{k}}})V 

The two most commonly used attention functions are additive attention and dot-product attention.

 MultiHead(Q,K,V)=Concat(head1,...,headh)WO \\mathrm{MultiHead}(Q,K,V)=\\mathrm{Concat}(\\mathrm{head}\_{1},...,\\mathrm{head}\_{h})W^{O}