#![allow(dead_code, non_snake_case)]
use dace::ast::Node;
use dace::ast::Stmt;
use dace::loop_node;
use std::rc::Rc;

pub fn lu(n: usize) -> Rc<Node> {
    let ubound = n as i32;
    let mut ref_a_ij = Node::new_ref("A", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut ref_a_ik = Node::new_ref("A", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut ref_a_kj = Node::new_ref("A", vec![n, n], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let mut ref_a_jj = Node::new_ref("A", vec![n, n], |ijk| {
        vec![ijk[1] as usize, ijk[1] as usize]
    });

    let mut k_loop_ref_j = loop_node!("k", 0 => move |ijk:&[i32]| ijk[1]);
    Node::extend_loop_body(&mut k_loop_ref_j, &mut ref_a_ik);
    Node::extend_loop_body(&mut k_loop_ref_j, &mut ref_a_kj);
    Node::extend_loop_body(&mut k_loop_ref_j, &mut ref_a_ij);

    let mut j_loop_lower_ref = loop_node!("j", 0 => move |ijk:&[i32]| ijk[0]);
    Node::extend_loop_body(&mut j_loop_lower_ref, &mut k_loop_ref_j);
    Node::extend_loop_body(&mut j_loop_lower_ref, &mut ref_a_jj);
    Node::extend_loop_body(&mut j_loop_lower_ref, &mut ref_a_ij);

    let mut k_loop_ref_i = loop_node!("k", 0 => move |ijk:&[i32]| ijk[0]);
    Node::extend_loop_body(&mut k_loop_ref_i, &mut ref_a_ik);
    Node::extend_loop_body(&mut k_loop_ref_i, &mut ref_a_kj);
    Node::extend_loop_body(&mut k_loop_ref_i, &mut ref_a_ij);

    let mut j_loop_upper_ref = loop_node!("j", move |ijk:&[i32]| ijk[0] => ubound);
    Node::extend_loop_body(&mut j_loop_upper_ref, &mut k_loop_ref_i);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_lower_ref);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_upper_ref);

    i_loop_ref
}

fn trmm_trace(M: usize, N: usize) -> Rc<Node> {
    let mut i_loop_ref = Node::new_single_loop("i", 0, M as i32);
    let mut j_loop_ref = Node::new_single_loop("j", 0, N as i32);
    let mut k_loop_ref =
        Node::new_single_loop("k", Node::get_lb(&i_loop_ref).unwrap() + 1, M as i32);

    // B[i * N + j] += A[k * M + i] * B[k * N + j];
    let mut a_ref = Node::new_ref("A", vec![N, M], |ijk| {
        vec![ijk[2] as usize, ijk[0] as usize]
    });
    let mut b1_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let mut b2_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    Node::extend_loop_body(&mut k_loop_ref, &mut a_ref);
    Node::extend_loop_body(&mut k_loop_ref, &mut b1_ref);
    Node::extend_loop_body(&mut k_loop_ref, &mut b2_ref);

    // B[i * N + j] = alpha * B[i * N + j];
    let mut b3_ref = Node::new_ref("B", vec![M, N], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    Node::extend_loop_body(&mut j_loop_ref, &mut b3_ref);
    Node::extend_loop_body(&mut j_loop_ref, &mut k_loop_ref);

    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);

    i_loop_ref
}

pub fn mvt(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating x1[i] = x1[i] + a[i][j] * y1[j];
    let mut s_ref_x1: Rc<Node> = Node::new_ref("x1", vec![n], |ij| vec![ij[0] as usize]);
    let mut s_ref_a1 = Node::new_ref("a1", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);
    let mut s_ref_y1 = Node::new_ref("y1", vec![n], |ij| vec![ij[1] as usize]);

    // creating loop j = 0, n { s_ref }
    let mut j_loop_ref = Node::new_single_loop("j", 0, ubound);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_x1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_a1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_y1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_x1);

    // creating loop i = 0, n
    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);

    //x2[i] = x2[i] + a[j][i] * y2[j];
    let mut s_ref_x2: Rc<Node> = Node::new_ref("x2", vec![n], |ij| vec![ij[0] as usize]);
    let mut s_ref_a2 = Node::new_ref("a2", vec![n, n], |ij| vec![ij[1] as usize, ij[0] as usize]);
    let mut s_ref_y2 = Node::new_ref("y2", vec![n], |ij| vec![ij[1] as usize]);

    // creating loop k = 0, n { s_ref }
    let mut k_loop_ref = Node::new_single_loop("k", 0, ubound);
    Node::extend_loop_body(&mut k_loop_ref, &mut s_ref_x2);
    Node::extend_loop_body(&mut k_loop_ref, &mut s_ref_a2);
    Node::extend_loop_body(&mut k_loop_ref, &mut s_ref_y2);
    Node::extend_loop_body(&mut k_loop_ref, &mut s_ref_x2);

    // creating loop m = 0, n
    let mut m_loop_ref = Node::new_single_loop("m", 0, ubound);
    Node::extend_loop_body(&mut m_loop_ref, &mut k_loop_ref);

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, m_loop_ref]))
}

pub fn trisolv(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating x[i] = b[i];
    let mut s_ref_x1 = Node::new_ref("x", vec![n], |ij| vec![ij[0] as usize]);
    let mut s_ref_b = Node::new_ref("b", vec![n], |ij| vec![ij[0] as usize]);

    // creating x[i] -= L[i][j] * x[j];
    let mut s_ref_L1 = Node::new_ref("L", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);
    let mut s_ref_x2 = Node::new_ref("x", vec![n], |ij| vec![ij[1] as usize]);
    let mut s_ref_x3 = Node::new_ref("x", vec![n], |ij| vec![ij[0] as usize]);

    // creating x[i] = x[i] / L[i][i]
    let mut s_ref_L2 = Node::new_ref("L", vec![n, n], |ij| vec![ij[0] as usize, ij[0] as usize]);
    // s_ref_x1

    let mut j_loop_ref = Node::new_single_loop_dyn_ub("j", 0, move |i| i[0]);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_L1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_x2);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_x3);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_x3);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_b);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_x1);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_x1);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_L2);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_x1);

    i_loop_ref
}

pub fn syrk(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    //creating C[i][j] = C[i][j] * beta
    let mut s_ref_c1 = Node::new_ref("c", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating C[i][j] = C[i][j] + alpha * A[i][k] * A[j][k]
    let mut s_ref_a1 = Node::new_ref("a1", vec![n, m], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut s_ref_a2 = Node::new_ref("a2", vec![n, m], |ijk| {
        vec![ijk[1] as usize, ijk[2] as usize]
    });
    let mut s_ref_c2 = Node::new_ref("c", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let mut j_loop_ref = Node::new_single_loop("j", 0, ubound1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_c1);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_c1);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound1);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);

    let mut m_loop_ref = Node::new_single_loop("m", 0, ubound2);
    Node::extend_loop_body(&mut m_loop_ref, &mut s_ref_a1);
    Node::extend_loop_body(&mut m_loop_ref, &mut s_ref_a2);
    Node::extend_loop_body(&mut m_loop_ref, &mut s_ref_c2);
    Node::extend_loop_body(&mut m_loop_ref, &mut s_ref_c2);

    let mut l_loop_ref = Node::new_single_loop("l", 0, ubound1);
    Node::extend_loop_body(&mut l_loop_ref, &mut m_loop_ref);

    let mut k_loop_ref = Node::new_single_loop("k", 0, ubound1);
    Node::extend_loop_body(&mut k_loop_ref, &mut l_loop_ref);

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, k_loop_ref]))
}

pub fn syr2d(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    // creating C[i][j] *= beta;
    let mut s_ref_c = Node::new_ref("c", vec![n, n], |ij| vec![ij[0] as usize, ij[1] as usize]);

    // creating C[i][j] += A[j][k]*alpha*B[i][k] + B[j][k]*alpha*A[i][k];
    let mut s_ref_a1 = Node::new_ref("a1", vec![n, m], |ijkl| {
        vec![ijkl[3] as usize, ijkl[2] as usize]
    });
    let mut s_ref_b1 = Node::new_ref("b1", vec![n, m], |ijkl| {
        vec![ijkl[0] as usize, ijkl[2] as usize]
    });
    let mut s_ref_b2 = Node::new_ref("b2", vec![n, m], |ijkl| {
        vec![ijkl[3] as usize, ijkl[2] as usize]
    });
    let mut s_ref_a2 = Node::new_ref("a2", vec![n, m], |ijkl| {
        vec![ijkl[0] as usize, ijkl[2] as usize]
    });
    let mut s_ref_c1 = Node::new_ref("c1", vec![n, n], |ijkl| {
        vec![ijkl[0] as usize, ijkl[3] as usize]
    });
    let mut s_ref_c2 = Node::new_ref("c2", vec![n, n], |ijkl| {
        vec![ijkl[0] as usize, ijkl[3] as usize]
    });

    let mut l_loop_ref = loop_node!("l", 0 => |i : &[i32]| i[0]);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_a1);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_b1);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_b2);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_a2);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_c1);
    Node::extend_loop_body(&mut l_loop_ref, &mut s_ref_c2);

    let mut k_loop_ref = Node::new_single_loop("k", 0, ubound2);
    Node::extend_loop_body(&mut k_loop_ref, &mut l_loop_ref);

    let mut j_loop_ref = loop_node!("j", 0 => |i : &[i32]| i[0]);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_c);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_c);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound1);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);
    Node::extend_loop_body(&mut i_loop_ref, &mut k_loop_ref);

    i_loop_ref
}

fn gemm(n: usize) -> Rc<Node> {
    let ubound = n as i32;

    let mut A0 = Node::new_ref("A0", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut B0 = Node::new_ref("B0", vec![n, n], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let mut C0 = Node::new_ref("C0", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut C1 = Node::new_ref("C1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let mut k_loop_ref = loop_node!("k", 0 => ubound);
    Node::extend_loop_body(&mut k_loop_ref, &mut A0);
    Node::extend_loop_body(&mut k_loop_ref, &mut B0);
    Node::extend_loop_body(&mut k_loop_ref, &mut C0);
    Node::extend_loop_body(&mut k_loop_ref, &mut C1);

    let mut C2 = Node::new_ref("C2", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut C3 = Node::new_ref("C3", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let mut j_loop_ref = loop_node!("j", 0 => ubound);
    Node::extend_loop_body(&mut j_loop_ref, &mut C2);
    Node::extend_loop_body(&mut j_loop_ref, &mut C3);
    Node::extend_loop_body(&mut j_loop_ref, &mut k_loop_ref);

    let mut i_loop_ref = loop_node!("i", 0 => ubound);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);

    i_loop_ref
}

fn _2mm(NI: usize, NJ: usize, NK: usize, NL: usize) -> Rc<Node> {
    let mut s_ref_tmp = Node::new_ref("tmp", vec![NI, NJ], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut s_ref_a = Node::new_ref("a", vec![NI, NK], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut s_ref_b = Node::new_ref("b", vec![NK, NJ], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let mut s_ref_c = Node::new_ref("c", vec![NL, NJ], |ijk| {
        vec![ijk[3] as usize, ijk[1] as usize]
    });
    let mut s_ref_d = Node::new_ref("d", vec![NI, NL], |ijk| {
        vec![ijk[0] as usize, ijk[3] as usize]
    });

    let mut knk_loop_ref = Node::new_single_loop("k", 0, NK as i32);
    let mut knk_loop_ref_clone = knk_loop_ref.clone();
    Node::extend_loop_body(&mut knk_loop_ref, &mut s_ref_a);
    Node::extend_loop_body(&mut knk_loop_ref, &mut s_ref_b);
    Node::extend_loop_body(&mut knk_loop_ref, &mut s_ref_tmp);

    let mut jnj_loop_ref = Node::new_single_loop("j", 0, NJ as i32);
    Node::extend_loop_body(&mut knk_loop_ref, &mut s_ref_tmp);
    Node::extend_loop_body(&mut knk_loop_ref, &mut knk_loop_ref_clone);

    let mut ini_loop_ref1 = Node::new_single_loop("i", 0, NI as i32);
    Node::extend_loop_body(&mut ini_loop_ref1, &mut jnj_loop_ref);

    let mut knj_loop_ref = Node::new_single_loop("k", 0, NJ as i32);
    Node::extend_loop_body(&mut knj_loop_ref, &mut s_ref_tmp);
    Node::extend_loop_body(&mut knj_loop_ref, &mut s_ref_c);
    Node::extend_loop_body(&mut knj_loop_ref, &mut s_ref_d);

    let mut jnl_loop_ref = Node::new_single_loop("j", 0, NL as i32);
    Node::extend_loop_body(&mut jnj_loop_ref, &mut s_ref_d);
    Node::extend_loop_body(&mut jnj_loop_ref, &mut knj_loop_ref);

    let mut ini_loop_ref2 = Node::new_single_loop("i", 0, NI as i32);
    Node::extend_loop_body(&mut ini_loop_ref2, &mut jnl_loop_ref);

    Node::new_node(Stmt::Block(vec![ini_loop_ref1, ini_loop_ref2]))
}

pub fn cholesky(n: usize) -> Rc<Node> {
    let ubound = n as i32;

    //create A[i * N + j] -= A[i * N + k] * A[j * N + k];
    let mut s_ref_aij1 = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut s_ref_aik1 = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut s_ref_ajk = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[1] as usize, ijk[2] as usize]
    });

    // create A[i * N + j] /= A[j * N + j];
    let mut s_ref_aij2 = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let mut s_ref_ajj = Node::new_ref("a", vec![n], |ijk| vec![ijk[1] as usize]);

    //create A[i * N + i] -= A[i * N + k] * A[i * N + k];
    let mut s_ref_aii1 = Node::new_ref("a", vec![n], |ijk| vec![ijk[0] as usize]);
    let mut s_ref_aik2 = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });

    //create A[i * N + i] = sqrt(A[i * N + i]);
    let mut s_ref_aii2 = Node::new_ref("a", vec![n], |ijk| vec![ijk[0] as usize]);

    let mut k1_loop_ref = Node::new_single_loop_dyn_ub("k", 0, move |j| j[0]);
    Node::extend_loop_body(&mut k1_loop_ref, &mut s_ref_aik1);
    Node::extend_loop_body(&mut k1_loop_ref, &mut s_ref_ajk);
    Node::extend_loop_body(&mut k1_loop_ref, &mut s_ref_aij1);
    Node::extend_loop_body(&mut k1_loop_ref, &mut s_ref_aij1);

    let mut j_loop_ref = Node::new_single_loop_dyn_ub("j", 0, move |i| i[0]);
    Node::extend_loop_body(&mut j_loop_ref, &mut k1_loop_ref);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_ajj);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_aij2);
    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_aij2);

    //independent of k1 loop above, not in the same scope, they share a k variable name and A accesses elements using k for both loops
    let mut k2_loop_ref = Node::new_single_loop_dyn_ub("k", 0, move |i| i[0]);
    Node::extend_loop_body(&mut k2_loop_ref, &mut s_ref_aik2);
    Node::extend_loop_body(&mut k2_loop_ref, &mut s_ref_aik2);
    Node::extend_loop_body(&mut k2_loop_ref, &mut s_ref_aii1);
    Node::extend_loop_body(&mut k2_loop_ref, &mut s_ref_aii1);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    Node::extend_loop_body(&mut i_loop_ref, &mut j_loop_ref);
    Node::extend_loop_body(&mut i_loop_ref, &mut k2_loop_ref);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_aii2);
    Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_aii2);

    i_loop_ref
}

#[cfg(test)]
mod tests {
    use super::*;
    fn trmm_trace_test() {
        let M = 1024;
        let N = 1024;

        let ast = trmm_trace(M, N);
        assert_eq!(ast.node_count(), 7);
    }

    #[test]
    fn test_mvt() {
        assert_eq!(mvt(1024).node_count(), 13);
    }

    #[test]
    fn test_trisolv() {
        assert_eq!(trisolv(1024).node_count(), 11);
    }

    #[test]
    fn test_syrk() {
        assert_eq!(syrk(256, 256).node_count(), 12);
    }

    #[test]
    fn test_syr2d() {
        assert_eq!(syr2d(1024, 1024).node_count(), 12);
    }

    #[test]
    fn test_gemm() {
        assert_eq!(gemm(128).node_count(), 9);
    }

    #[test]
    fn _2mm_test() {
        assert_eq!(_2mm(1024, 1024, 1024, 1024).node_count(), 10);
    }

    #[test]
    fn lu_test() {
        let mm = lu(100);
        assert_eq!(mm.node_count(), 13);
    }

    #[test]
    fn test_cholesky() {
        assert_eq!(cholesky(1024).node_count(), 17)
    }
}
