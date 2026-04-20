// Liabilities type definitions

export interface LiabilityMetadata {
  liabilityType?:
    | "mortgage"
    | "auto_loan"
    | "student_loan"
    | "credit_card"
    | "personal_loan"
    | "heloc";
  linkedAssetId?: string;
  originalAmount?: string;
  originationDate?: string;
  interestRate?: string;
}

export interface LinkLiabilityRequest {
  /** ID of the property/vehicle to link to */
  targetAssetId: string;
}

export interface UnlinkLiabilityRequest {
  /** ID of the liability to unlink */
  liabilityId: string;
}
