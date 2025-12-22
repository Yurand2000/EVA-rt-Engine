# Multiprocessor Periodic Resource Model

[**🏠 Go back to all analyses**](../../../README.md#-available-analyses)

🚧 Under Construction 🚧

### 🧪 Implemented Analyses

---

- **Global EDF - Shin, Easwaran, Lee** (2008) [^1]

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: Pseudo-Polynomial in the input parameters.

- **Global EDF - Bertogna, Cirinei, Lipari demand** (2008) [^2]

    *same framework from Shin et al, buth with Bertogna et al __adapted__ demand function*

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: *O(n^2)*

- **Global FP - Bertogna, Cirinei, Lipari demand** (2008) [^2]

    *same framework from Shin et al, buth with Bertogna et al __adapted__ demand function*

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: *O(n^2)*

### ⚙️ Interface Generation

---

- **Global EDF - Shin, Easwaran, Lee** (2008) [^1]
- **Global EDF - Bertogna, Cirinei, Lipari demand** (2008) [^2]
- **Global FP - Bertogna, Cirinei, Lipari demand** (2008) [^2]

### 📚 References

[^1]: I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
[^2]: Bertogna, M., Cirinei, M. and Lipari, G., 2008. Schedulability analysis of global scheduling algorithms on multiprocessor platforms. IEEE Transactions on parallel and distributed systems, 20(4), pp.553-566.