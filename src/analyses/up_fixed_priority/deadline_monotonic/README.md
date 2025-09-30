# UniProcessor Deadline Monotonic

[**🏠 Go back to all analyses**](../../../README.md#-available-analyses)

This module checks if the **Deadline Monotonic** priority assignment for **Fixed-Priority Scheduling** is suitable for the given taskset on a **UniProcessor** System.

### 🔑 Preconditions

---

- Taskset sorted by deadlines.
- Constrained deadlines.

### 🧪 Implemented Analyses

---

- **Leung & Whitehead** (1982) [^1]

### 📚 References

[^1]: J. Y.-T. Leung and J. Whitehead, “On the complexity of fixed-priority scheduling of periodic, real-time tasks,” Performance Evaluation, vol. 2, no. 4, pp. 237–250, Dec. 1982, doi: 10.1016/0166-5316(82)90024-4.