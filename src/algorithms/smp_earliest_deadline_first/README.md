# SMP Global Earliest Deadline First

[**🏠 Go back to all analyses**](../../../README.md#-available-analyses)

### 🧪 Implemented Analyses

---

- **GFB Periodic-Implicit** - Goossens, Funk and Baruah (2003) [^1]

    Preconditions:
    - Periodic Tasks
    - Implicit deadlines

    Worst-Case Complexity: *O(n)*

- **GFB Sporadic-Constrained** - Bertogna, Cirinei and Lipari (2005) [^3]

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: *O(n)*

- **BAK Test** - Baker (2003) [^4]

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: *O(n^3)*

- **BCL-EDF** - Bertogna, Cirinei and Lipari (2005) [^3]

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: *O(n^2)*

- **Baruah** - Baruah (2007) [^5]

    Preconditions:
    - Sporadic Tasks
    - Constrained deadlines

    Worst-Case Complexity: Pseudo-Polynomial *O(n \* m)* where *n* is the number of tasks and *m* is the number of processors.

### 📚 References

[^1]: J. Goossens, S. Funk, and S. Baruah, “Priority-Driven Scheduling of Periodic Task Systems on Multiprocessors,” Real-Time Systems, vol. 25, no. 2, pp. 187–205, Sept. 2003, doi: 10.1023/A:1025120124771.
[^2]: M. Bertogna, M. Cirinei, and G. Lipari, “Schedulability Analysis of Global Scheduling Algorithms on Multiprocessor Platforms,” IEEE Transactions on Parallel and Distributed Systems, vol. 20, no. 4, pp. 553–566, Apr. 2009, doi: 10.1109/TPDS.2008.129.
[^3]: M. Bertogna, M. Cirinei, and G. Lipari, “Improved schedulability analysis of EDF on multiprocessor platforms,” in 17th Euromicro Conference on Real-Time Systems (ECRTS’05), July 2005, pp. 209–218. doi: 10.1109/ECRTS.2005.18.
[^4]: T. P. Baker, “Multiprocessor EDF and deadline monotonic schedulability analysis,” in RTSS 2003. 24th IEEE Real-Time Systems Symposium, 2003, Dec. 2003, pp. 120–129. doi: 10.1109/REAL.2003.1253260.
[^5]: S. Baruah, “Techniques for Multiprocessor Global Schedulability Analysis,” in 28th IEEE International Real-Time Systems Symposium (RTSS 2007), Tucson, AZ, USA: IEEE, Dec. 2007, pp. 119–128. doi: 10.1109/RTSS.2007.35.