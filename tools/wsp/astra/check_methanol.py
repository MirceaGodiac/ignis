"""Analytic, no-fit audit of the methanol self-cooling evaporation estimate.

This script reads no experiment, split, release, or model file. It evaluates the two
isothermal d-squared laws written in docs/wsp/research/phys_flex_methanol.md and
docs/wsp/research/properties_verified.md at listed temperatures. These are physics
checks, not model weights or train measurements.
"""

from math import log

R = 8.314462618  # J mol-1 K-1
P_ATM_PA = 101325.0
P_ATM_BAR = 1.01325
MW_F = 0.032042  # kg mol-1, methanol
MW_N2 = 0.028013  # kg mol-1, nitrogen
D300_M2_S = 0.153e-4  # methanol-N2, cm2/s at 300 K converted to m2/s
D_EXP = 1.78
RHO_L_300 = 784.5  # kg/m3
DRHO_DT = -0.96  # kg/m3/K
ANTOINE_MIN_K = 288.1
ANTOINE_MAX_K = 356.83


def p_sat_bar(temp_k: float) -> float:
    # NIST Antoine form valid here: 288.1–356.83 K.
    return 10 ** (5.20409 - 1581.341 / (temp_k - 33.50))


def evaporation_rates(temp_k: float) -> tuple[float, float, float]:
    """Return (x_surface, ambient-density K, exact-molar K), all K in mm2/s."""
    x_surface = p_sat_bar(temp_k) / P_ATM_BAR
    d_f = D300_M2_S * (temp_k / 300.0) ** D_EXP
    rho_g = P_ATM_PA * MW_N2 / (R * temp_k)
    rho_l = RHO_L_300 + DRHO_DT * (temp_k - 300.0)

    # The dossier's classical ambient-density expression.
    y_surface = x_surface * MW_F / (x_surface * MW_F + (1 - x_surface) * MW_N2)
    b_mass = y_surface / (1 - y_surface)
    k_ambient = 8 * rho_g * d_f / rho_l * log(1 + b_mass)

    # Molar Stefan-flow form, as stated in properties_verified.md.
    c_g = P_ATM_PA / (R * temp_k)
    k_molar = 8 * MW_F * c_g * d_f / rho_l * log(1 / (1 - x_surface))
    return x_surface, k_ambient * 1e6, k_molar * 1e6


def main() -> None:
    print("T_K  Antoine_status  x_surface  K_ambient_mm2_s  K_molar_mm2_s")
    for temp_k in (280.0, 290.0, 300.0, 310.0, 330.0, 335.0):
        x, k_ambient, k_molar = evaporation_rates(temp_k)
        status = "in-range" if ANTOINE_MIN_K <= temp_k <= ANTOINE_MAX_K else "extrapolated"
        print(f"{temp_k:5.1f}  {status:12s}  {x:.6f}  {k_ambient:.5f}  {k_molar:.5f}")
    x330, _, k330 = evaporation_rates(330.0)
    x335, _, k335 = evaporation_rates(335.0)
    print(f"K(335)/K(330)={k335 / k330:.4f}")
    print(f"molar correction at 335 K={(k335 / evaporation_rates(335.0)[1] - 1) * 100:.2f}%")
    print(f"x_surface(335 K)={x335:.6f}; x_surface(330 K)={x330:.6f}")


if __name__ == "__main__":
    main()
