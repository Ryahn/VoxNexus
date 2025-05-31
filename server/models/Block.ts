import mongoose, { Schema, Document, Model, Types } from 'mongoose'

export interface IBlock extends Document {
  blockerId: Types.ObjectId
  blockedId: Types.ObjectId
  createdAt: Date
}

const BlockSchema: Schema<IBlock> = new Schema({
  blockerId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  blockedId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  createdAt: { type: Date, default: Date.now },
})

export default (mongoose.models.Block as Model<IBlock>) || mongoose.model<IBlock>('Block', BlockSchema) 